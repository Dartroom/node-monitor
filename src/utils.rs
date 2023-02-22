use anyhow::Result;
use chrono::prelude::*;
use config::{Config, File, FileFormat};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    io::{Read, Seek},
};
use tokio::time::{self, Duration};

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

pub fn get_settings() -> Result<MonitorSettings> {
    // check if the config.json file exists if not, create a new

    let settings = Config::builder()
        .add_source(File::new("settings.json", FileFormat::Json5))
        .build()?;

    let result = settings.try_deserialize::<MonitorSettings>()?;
    //println!("{:#?}", result);
    Ok(result)
}

pub fn set_interval<F, Fut>(mut f: F, dur: Duration)
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

pub async fn fetch_data(settings: &MonitorSettings) -> Result<()> {
    use std::fs::File;
    // add access Token;
    let mut headers = header::HeaderMap::new();
    if !settings.x_algo_token.is_empty() {
        let val = header::HeaderValue::from_str(&settings.x_algo_token)?;
        headers.insert("X-Algo-API-Token", val);
    }

    // use the config setting to fetch the remote node;
    let client = Client::builder().default_headers(headers).build()?;
    println!("fetching from local node: {}", settings.local_node);

    let local = match get_data(&settings.local_node, &client).await {
        Ok(local) => local,
        Err(e) => {
            // error, update the setting to stopped;
            let mut store = load_from_store()?;

            store.status = Status::Stopped;
            // store.local_last_round = store.local_last_round;

            // update the time when file is out of sync;
            let time = Utc::now();
            store.time_stamp = Some(time.to_string());
            let mut f = File::create("data.json")?;
            serde_json::to_writer_pretty(f, &store)?;

            let mut resp = NodeResponse::default();
            resp.last_round += store.local_last_round;

            resp
        }
    };
    println!("fetching from remote node: {}", settings.cluster_nodes[0]);
    // we  want to fetch data from all remote nodes, the chosen one is the one with highest last_round;
    let remote_nodes: Vec<_> = settings
        .cluster_nodes
        .iter()
        .map(|s| async {
            get_data(s, &client).await;
        })
        .collect();
    let remote: NodeResponse = get_data(&settings.cluster_nodes[0], &client).await?;

    let local_data = save_data(local, remote);

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

pub fn save_data(resp: NodeResponse, remote_res: NodeResponse) -> Result<Payload> {
    use std::fs;
    use std::path::Path;
    //  read the process directory and check for data.json file;
    let data_file_exists = Path::new("data.json").exists();

    if (!data_file_exists) {
        println!("no data.json file, creatting one");
        let mut f = fs::File::create("data.json")?;
        serde_json::to_writer_pretty(f, &Payload::default())?;
    }

    let settings = Config::builder()
        .add_source(File::new("data.json", FileFormat::Json))
        .build()?;

    let mut store = settings.try_deserialize::<Payload>()?;

    // if both the store data and local node data is the same, we update the local data;
    if resp.last_round == store.local_last_round {
        // if the file data is the same after a round, we update
        store.status = Status::Stopped;
        store.offset = remote_res.last_round - store.local_last_round;
        store.local_last_round = remote_res.last_round;
        // update the time when file is out of sync;
        let time = Utc::now();
        store.time_stamp = Some(time.to_string());

        // do nothing  the data is synced with the remote
    } else {
        // if the data is different, save the local;
        //store.status = Status::Stopped;
        if remote_res.last_round == resp.last_round {
            store.status = Status::Synced;
            store.offset = remote_res.last_round - resp.last_round;
            store.time_stamp = None;
            store.remote_last_round = remote_res.last_round;
        } else {
            store.status = Status::CatchingUp;
            store.offset = remote_res.last_round - resp.last_round;
            let time = Utc::now();
            store.time_stamp = Some(time.to_string());
        }

        store.local_last_round = resp.last_round;
        store.time_since_last = resp.time_since_last_round;
        store.remote_last_round = remote_res.last_round;
        // when node starts sync
        // write the change to the out data;

        // update the data file
        //let now = time::Instant::now();
        let mut f = fs::File::create("data.json")?;
        serde_json::to_writer_pretty(f, &store)?;
        //println!("{:?}", now.elapsed());
    }

    Ok(store)
}

// load the data from store;
pub fn load_from_store() -> Result<Payload> {
    let settings = Config::builder()
        .add_source(File::new("data.json", FileFormat::Json5))
        .build()?;

    let mut store = settings.try_deserialize::<Payload>()?;
    Ok(store)
}
