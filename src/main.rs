#![allow(dead_code, unused)]
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder,HttpRequest};
use slog::Logger;
mod logger;
mod utils;
use logger::*;

#[get("/")]
async fn hello(
    log: web::Data<Logger>,
    config: web::Data<utils::MonitorSettings>,
) -> impl Responder {
    info!(log, "{config:#?}");

    HttpResponse::Ok().json(config)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let log = logger::configure_log();
    let config = utils::get_settings();
    let port = config.port;
    //println!("hello {:#?}", config);

    utils::set_interval(
        move || async {
            println!("fetching data every 10 seconds");
        },
        std::time::Duration::from_secs(config.polling_rate),
    );

    info!(log, "Starting the server at http://127.0.0.1:{port}/");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(log.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(hello)
    })
    .bind(("127.0.0.1", port as u16))?
    .run()
    .await
}
