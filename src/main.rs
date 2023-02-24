#![allow(dead_code, unused, non_snake_case, non_camel_case_types)]
use actix_web::{
    middleware::{Compress, Logger},
    web, App, HttpServer,
};
use serde::{Deserialize, Serialize};
use std::fmt::format;
mod logger;
mod polling;
mod utils;

use logger::*;
mod cli;
mod response;
use cli::*;
use polling::*;
use response::*;
#[macro_use]
extern crate lazy_static;
// import a mutex

// setup cli tool using clap

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let log_level = if ARGS.clone().verbose {
        "trace"
    } else {
        "info"
    };

    //std::env::set_var("RUST_LOG", "node_monitor=info");
    //std::env::set_var("RUST_TRACE", "1");
    //env_logger::init();

    env_logger::init_from_env(
        env_logger::Env::new()
            .default_filter_or(&format!("actix_web={log_level},node_monitor={log_level}")),
    );
    // println!("{:?}", args);
    //let log = LOGGER.clone();

    //println!("{:#?}", &path);
    let c = utils::get_settings(ARGS.clone().config);
    //let args: Vec<_> = std::env::args().collect();
    if c.is_err() {
        let err = c.unwrap_err();

        //eprintln!("{:?}", err.source());
        error!("{:?}", err);
        Ok(())
    } else {
        let config = c.unwrap();
        //let now: Mutex<Instant> = Mutex::new(Instant::now());
        let port = config.port;
        // polling  for config.polling_rate
        poll(&config);

        info!("Starting the server at http://127.0.0.1:{port}/health");
        let dir_path = ARGS.clone().data_dir;
        //info!("{dir_path:?}");
        //let log = logger::configure_log();
        HttpServer::new(move || {
            App::new()
                //.app_data(web::Data::new(log.clone()))
                .app_data(web::Data::new(config.clone()))
                .app_data(web::Data::new(dir_path.clone()))
                .wrap(Logger::default())
                .service(get_health)
        })
        .bind(("127.0.0.1", port))?
        .run()
        .await
    }
}
