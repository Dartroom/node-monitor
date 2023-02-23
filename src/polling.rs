use crate::cli::*;
use crate::logger::*;
use crate::utils::*;
use crate::utils::*;

pub fn poll(config: &MonitorSettings) {
    set_interval(
        move || async move {
            // send a request to our node to check status;
            // Check the last round value to see if it increased;
            let args = ARGS.clone();
            let path = args.config;
            let config = get_settings(path).expect("failed to get settings");
            let log = LOGGER.clone();
            //let n =  now.lock();

            fetch_data(&config, args.data_dir)
                .await
                .expect("failed to fron nodes");
            info!(
                log,
                "fetching data every {:?} seconds...", config.polling_rate
            );
            // println!("{:?}", utils::get::<String>("foo"));
        },
        std::time::Duration::from_secs(config.polling_rate),
    );
}
