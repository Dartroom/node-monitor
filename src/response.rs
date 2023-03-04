#![allow(dead_code, unused, non_snake_case)]
use crate::cli::*;
use crate::logger::*;
use crate::utils::*;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use slog::Logger;

pub async fn get_health(
    //log: web::Data<Logger>,
    //config: web::Data<MonitorSettings>,
    //_resp: HttpRequest,
    dir_arg: web::Data<Option<String>>,
) -> impl Responder {
    use Status::*;
    //info!(log, "{config:#?}");
    // read the rest
    let path = dir_arg.into_inner();
    let dir_path = path.as_deref().unwrap_or("");
    let path_dir = if dir_path.is_empty() {
        None
    } else {
        Some(dir_path.to_string())
    };
      //println!( "{:?}", path_dir );

    let payload = load_from_store(path_dir).unwrap();
     //println!( "{:?}", payload);
    // get the status
    match payload.status {
        Synced => {
            info!( " local node is synced");
            HttpResponse::Ok().json(payload)
        }
        Stopped => {
            info!( " local node is stopped syncing ");
            // return statusCode of 503

            HttpResponse::ServiceUnavailable().json(payload)
        }
        CatchingUp => {
            info!( " local node is caughing up");
            HttpResponse::ServiceUnavailable().json(payload)
        }
    }
}
