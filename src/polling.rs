use crate::cli::*;
use crate::logger::*;
use crate::utils::*;
use crate::utils::*;
use anyhow::*;
pub  async fn poll(config: &MonitorSettings) {
    set_interval(
        move || async {
            // send a request to our node to check status;
            /// Check the last round value to see if it increased;
            let args = ARGS.clone();
            let path = args.config;
            //let log = LOGGER.clone();
            let config = get_settings(path.clone())
                .with_context(|| format!("Failed to read configuration settings from {path:?}"));

            //let n =  now.lock();
            let result = config.unwrap();
            fetch_data(&result, args.data_dir)
                .await
                .with_context(|| "fetching error");
            info!(
            
                "fetching data every {:?} seconds...", &result.polling_rate
            );
            // println!("{:?}", utils::get::<String>("foo"));
        },
        std::time::Duration::from_secs(config.polling_rate),
    ).await
}
