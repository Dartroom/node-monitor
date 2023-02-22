#![allow(dead_code, unused)]
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use slog::Logger;
mod logger;
mod utils;
use anyhow::Result;
use logger::*;
// import a mutex
use std::{sync::Mutex, time::Instant};

#[get("/health")]
async fn get_health(
    log: web::Data<Logger>,
    config: web::Data<utils::MonitorSettings>,
) -> impl Responder {
    use utils::Status::*;
    //info!(log, "{config:#?}");
    // read the rest
    let payload = utils::load_from_store().unwrap();
    // get the status
    match payload.status {
        Synced => {
            info!(log, " local node is synced");
            HttpResponse::Ok().json(payload)
        }
        Stopped => {
            info!(log, " local node is stopped syncing ");
            // return statusCode of 503

            HttpResponse::ServiceUnavailable().json(payload)
        }
        CatchingUp => {
            info!(log, " local node is caught up");
            HttpResponse::ServiceUnavailable().json(payload)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let log = logger::configure_log();
    let path = std::env::args().nth(1);
    println!("{:#?}", &path);
    let config = utils::get_settings(path.clone()).expect("failed to parse configuration file");
    let args: Vec<_> = std::env::args().collect();

    //let now: Mutex<Instant> = Mutex::new(Instant::now());

    let port = config.port;
    //println!("hello {:#?}", config);

    utils::set_interval(
        move || async move {
            // send a request to our node to check status;
            // Check the last round value to see if it increased;
            let path = std::env::args().nth(1);
            let config = utils::get_settings(path).expect("failed to get settings");
            //let n =  now.lock();

            utils::fetch_data(&config)
                .await
                .expect("failed to fron nodes");
            println!("fetching data every {:?} seconds...", config.polling_rate);
            // println!("{:?}", utils::get::<String>("foo"));
        },
        std::time::Duration::from_secs(config.polling_rate),
    );

    info!(log, "Starting the server at http://127.0.0.1:{port}/health");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(log.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(get_health)
    })
    .bind(("127.0.0.1", port as u16))?
    .run()
    .await
}
