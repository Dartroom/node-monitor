#![allow(dead_code, unused, non_snake_case)]
use crate::cli::*;
use crate::logger::*;
use crate::utils::*;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use slog::Logger;
#[get("/health")]

pub async fn get_health(
    log: web::Data<Logger>,
    config: web::Data<MonitorSettings>,
    dir_arg: Option<String>,
) -> impl Responder {
    use Status::*;
    //info!(log, "{config:#?}");
    // read the rest
    //let path = dir_arg.as_ref();
    let payload = load_from_store(dir_arg).unwrap();
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
