use crate::logger::*;
use anyhow::{Context, Result};
use chrono::prelude::*;
use config::{Config, File, FileFormat};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::future::Future;

use tokio::time::{self, Duration};

use futures::future::join_all;
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MonitorSettings {
    pub polling_rate: u64,
    pub x_algo_token: String, // token API key
    pub valid_round_range: i64,
    pub local_node: String,         // remote address of the some remote
    pub cluster_nodes: Vec<String>, // the other node;
    pub port: u64,
    pub node_port: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Status {
    #[default]
    Synced,
    Stopped,
    CatchingUp,
}
// Response from the node;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeResponse {
    /*   pub catchpoint: String,
    //#[serde(rename = "catchpoint-acquired-blocks")]
    pub catchpoint_acquired_blocks: i64,
    //#[serde(rename = "catchpoint-processed-accounts")]
    pub catchpoint_processed_accounts: i64,
    //#[serde(rename = "catchpoint-processed-kvs")]
    //pub catchpoint_processed_kvs: i64,
    #[serde(rename = "catchpoint-total-accounts")]
    pub catchpoint_total_accounts: i64,
    #[serde(rename = "catchpoint-total-blocks")]
    pub catchpoint_total_blocks: i64,
    #[serde(rename = "catchpoint-total-kvs")]
    pub catchpoint_total_kvs: i64,
    #[serde(rename = "catchpoint-verified-accounts")]
    pub catchpoint_verified_accounts: i64,
    #[serde(rename = "catchpoint-verified-kvs")]
    pub catchpoint_verified_kvs: i64,
    #[serde(rename = "catchup-time")]
    pub catchup_time: i64,
    #[serde(rename = "last-catchpoint")]
    pub last_catchpoint: String,
    */
    #[serde(rename = "last-round")]
    pub last_round: i64,
    #[serde(rename = "last-version")]
    pub last_version: String,
    #[serde(rename = "next-version")]
    pub next_version: String,
    #[serde(rename = "next-version-round")]
    pub next_version_round: i64,
    #[serde(rename = "next-version-supported")]
    pub next_version_supported: bool,
    #[serde(rename = "stopped-at-unsupported-round")]
    pub stopped_at_unsupported_round: bool,
    #[serde(rename = "time-since-last-round")]
    pub time_since_last_round: i64,
}

// our metrics

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub status: Status,             // synced or not
    pub time_stamp: Option<String>, // when stopped, when catching up
    pub local_last_round: i64,      //
    pub remote_last_round: i64,     // timestamp
    pub offset: i64,
    pub time_since_last: i64, // difference between updates
}

/* build the setting struct from  */

pub fn get_settings(path: Option<String>) -> anyhow::Result<MonitorSettings> {
    // check if the config.json file exists if not, create a new
    let file_path = if let Some(right) = path {
        right
    } else {
        "settings.json".to_string()
    };
    //println!("{:?}", file_path);
    let settings = Config::builder()
        .add_source(File::with_name(&file_path))
        .build()
        .with_context(|| format!("failed to load configuration file: {} ", &file_path))?;

    let result = settings.try_deserialize::<MonitorSettings>()?;
    //println!("{:#?}", result);
    Ok(result)
}

pub fn set_interval<F, Fut>(f: F, dur: Duration)
where
    F: Send + 'static + Fn() -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    // Create stream of intervals.
    let mut interval = time::interval(dur);

    tokio::spawn(async move {
        // Skip the first tick at 0ms.
        interval.tick().await;
        loop {
            // Wait until next tick:
            interval.tick().await;
            // Spawn a task for the operation.
            tokio::spawn(f());
        }
    });
}
// Throughout the settings structure -> create functions
// node-aws

pub async fn fetch_data(
    settings: &MonitorSettings,
    dir: Option<String>,
) -> Result<(), anyhow::Error> {
    use std::fs::File;
    // add access Token;
    let mut headers = header::HeaderMap::new();
    if !settings.x_algo_token.is_empty() {
        let val = header::HeaderValue::from_str(&settings.x_algo_token)?;
        headers.insert("X-Algo-API-Token", val);
    }

    // use the config setting to fetch the remote node;
    let client = Client::builder().default_headers(headers).build()?;

    // we  want to fetch data from all remote nodes, the chosen one is the one with highest last_round;
    let remote_nodes = join_all(settings.cluster_nodes.iter().map(|s| get_data(s, &client))).await;

    // use the most update node as the remote node;
    let most_update_node = remote_nodes
        .into_iter()
        .filter(|e| !e.is_err())
        .max_by_key(|x| x.as_ref().unwrap().last_round);
    let mut remote_connection = true;
    if let Some(node) = most_update_node {
        if node.is_ok() {
            let d = node.unwrap();
            info!("fetching from remote active remote");
            //let remote: NodeResponse = get_data(&, &client).await?;
            info!("fetching from local node: {}", settings.local_node);
            let mut local_connection = true;
            let local = match get_data(&settings.local_node, &client).await {
                Ok(local) => local,
                Err(e) => {
                    local_connection = false;
                    warn!(
                        "Failed to fetch from local node: {}, status changing",
                        settings.local_node
                    );
                    // error, update the setting to stopped;
                    let mut store = load_from_store(dir.clone())?;

                    store.status = Status::Stopped;
                    // store.local_last_round = store.local_last_round;
                    // use the remote_round from the remote node;
                    let off = d.last_round - store.local_last_round;
                    store.remote_last_round = d.last_round;
                    store.offset = off;
                    // update the time when file is out of sync;
                    let time = Utc::now();
                    store.time_stamp = Some(time.to_string());
                    let file_path = if let Some(right) = dir.clone() {
                        format!("{right}/{}", "data.json")
                    } else {
                        "data.json".to_string()
                    };
                    let mut f = File::create(file_path)?;
                    serde_json::to_writer_pretty(f, &store)?;

                    let mut resp = NodeResponse::default();
                    resp.last_round += store.local_last_round;
                    // set  the last_round to the remote last round

                    resp
                }
            };
            info!("local node is connected: {:?},{local:?}", local_connection);
            let local_data = save_data(local, d, settings, dir, local_connection);
        }
    }
    //

    //println!("{}", local.last_round == remote.last_round);
    //println!("{:#?}", local_data);

    Ok(())
}

async fn get_data(url: &str, client: &Client) -> Result<NodeResponse, reqwest::Error> {
    // use the config setting to fetch the remote node;
    //let client = Client::builder().default_headers(headers).build()?;
    let url = format!("{}{}", url, "/v2/status");
    //println!("{url}");
    client.get(url).send().await?.json().await
}

pub fn save_data(
    resp: NodeResponse,
    remote_res: NodeResponse,
    global: &MonitorSettings,
    dir: Option<String>,
    local_connection: bool,
) -> Result<Payload> {
    use std::fs;
    use std::path::Path;
    let file_path = if let Some(right) = dir {
        format!("{right}/{}", "data.json")
    } else {
        "data.json".to_string()
    };
    //  read the process directory and check for data.json file;
    //println!("{}", file_path);
    let data_file_exists = Path::new(&file_path).exists();

    if (!data_file_exists) {
        info!("no data.json file, creating one");
        let mut f = fs::File::create(&file_path)?;
        serde_json::to_writer_pretty(f, &Payload::default())?;
    }

    let settings = Config::builder()
        .add_source(File::with_name(&file_path))
        .build()?;

    let mut store = settings.try_deserialize::<Payload>()?;
    // use  the config setting to fetch the remotes;

    // if both the store data and local node data is the same, we update the local data;
    debug!("{} =={}", resp.last_round, store.local_last_round);
    if resp.last_round == store.local_last_round || !local_connection {
        let offset = store.offset;
        // if the file data is the same after a round, we update
        debug!(" localOffset: {offset}");
        if offset > global.valid_round_range {
            // update the time once
            if (store.time_stamp.is_none() && store.status != Status::Stopped) {
                let time = Utc::now();
                store.time_stamp = Some(time.to_string());
            }
            store.status = Status::Stopped;
            store.offset = remote_res.last_round - store.local_last_round;
            store.local_last_round = resp.last_round;
            // update the time when file is out of sync;

            store.time_since_last = resp.time_since_last_round;
            store.remote_last_round = remote_res.last_round;
            update_store(&store, file_path)?;
        } else {
            // dothin
        }
        // do nothing  the data is synced with the remote
    } else {
        // if the data is different, save the local;
        //store.status = Status::Stopped;
        debug!("{} =={}", resp.last_round, store.local_last_round);
        let offset = remote_res.last_round - resp.last_round;
        if remote_res.last_round == resp.last_round {
            store.status = Status::Synced;
            store.offset = offset;
            store.time_stamp = None;
            store.remote_last_round = remote_res.last_round;
            store.time_since_last = resp.time_since_last_round;
            // store.remote_last_round = remote_res.last_round;
            update_store(&store, file_path.clone())?;
        }
        if (resp.last_round > store.local_last_round && local_connection) {
            store.status = Status::CatchingUp;
            store.offset = remote_res.last_round - resp.last_round;
            //let time = Utc::now();
            // store.time_stamp = Some(time.to_string());
            store.time_stamp = None;
            store.local_last_round = resp.last_round;
            store.time_since_last = resp.time_since_last_round;
            store.remote_last_round = remote_res.last_round;

            update_store(&store, file_path)?;
        }

        // update the data file
        //let now = time::Instant::now();

        //println!("{:?}", now.elapsed());
    }

    Ok(store)
}

// load the data from store;
pub fn load_from_store(dir: Option<String>) -> Result<Payload> {
    // if directory path it p

    let file_path = if let Some(right) = dir {
        format!("{right}/{}", "data.json")
    } else {
        "data.json".to_string()
    };
    //println!("Loading data from:{file_path}");
    let settings = Config::builder()
        .add_source(File::with_name(&file_path))
        .build()?;

    let mut store = settings.try_deserialize::<Payload>()?;
    Ok(store)
}

fn update_store(store: &Payload, dir: String) -> Result<()> {
    use std::fs;
    let mut f = fs::File::create(dir)?;
    serde_json::to_writer_pretty(f, &store)?;
    Ok(())
}
