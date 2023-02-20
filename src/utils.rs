use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::future::Future;
use tokio::time::{self, Duration};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MonitorSettings {
    pub polling_rate: u64,
    pub x_algo_token: String,
    pub valid_round_range: i64,
    pub local_node: String,
    pub cluster_nodes: Vec<String>,
    pub port: u64,
    pub node_port: u64,
}

/* build the setting struct from  */

pub fn get_settings() -> MonitorSettings {
    let settings = Config::builder()
        .add_source(File::with_name("config.json"))
        .build()
        .unwrap();
    settings.try_deserialize::<MonitorSettings>().unwrap()
}

pub fn set_interval<F, Fut>(mut f: F, dur: Duration)
where
    F: Send + 'static + FnMut() -> Fut,
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
