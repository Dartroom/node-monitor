#![allow(dead_code, unused, non_snake_case)]
use actix_web::{web, App, HttpServer};
mod logger;
mod utils;

use logger::*;
mod cli;
mod response;
use cli::*;
use response::*;
#[macro_use]
extern crate lazy_static;
// import a mutex

// setup cli tool using clap

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // println!("{:?}", args);
    let log = LOGGER.clone();

    //println!("{:#?}", &path);
    let config =
        utils::get_settings(ARGS.clone().config).expect("failed to parse configuration file");
    //let args: Vec<_> = std::env::args().collect();

    //let now: Mutex<Instant> = Mutex::new(Instant::now());

    let port = config.port;
    //println!("hello {:#?}", config);

    utils::set_interval(
        move || async move {
            // send a request to our node to check status;
            // Check the last round value to see if it increased;
            let args = ARGS.clone();
            let path = args.config;
            let config = utils::get_settings(path).expect("failed to get settings");
            let log = LOGGER.clone();
            //let n =  now.lock();

            utils::fetch_data(&config, args.data_dir)
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

    info!(log, "Starting the server at http://127.0.0.1:{port}/health");
    HttpServer::new(move || {
        App::new()
            .app_data(LOGGER.clone())
            .app_data(web::Data::new(config.clone()))
            .app_data(ARGS.clone().data_dir)
            .service(get_health)
    })
    .bind(("127.0.0.1", port as u16))?
    .run()
    .await
}
