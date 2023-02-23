#![allow(dead_code, unused, non_snake_case)]
use actix_web::{web, App, HttpServer};
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
    // println!("{:?}", args);
    let log = LOGGER.clone();

    //println!("{:#?}", &path);
    let c = utils::get_settings(ARGS.clone().config);
    //let args: Vec<_> = std::env::args().collect();
    if c.is_err() {
        let err = c.unwrap_err();

        //eprintln!("{:?}", err.source());
        eprint!("{:?}", err);
        Ok(())
    } else {
        let config = c.unwrap();
        //let now: Mutex<Instant> = Mutex::new(Instant::now());
        let port = config.port;
        // polling  for config.polling_rate
        poll(&config);

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
}
