use actix_cors::Cors;
use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use db::prepare_connection;
use log::{debug, error};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

use crate::db::{insert_new_download, select_data, update_download};
use crate::url::check_url;

mod db;
mod downloader;
mod url;

const DB_FILE: &str = "./config/file-downloader.db";
const DOWNLOAD_DIR: &str = "./downloads/";
const FILES_DIR: &str = "../front/dist/";

#[derive(Deserialize, Debug)]
struct DownloadQuery {
    download_url: String,
}

#[derive(Debug)]
pub struct DownloadResult {
    pub file_name: String,
    pub file_size: u64,
}

struct ServerState {
    db_conn: Arc<Mutex<Connection>>,
}

#[get("/health")]
async fn health_check() -> impl Responder {
    "Healthy"
}

#[post("/download")]
async fn download(
    state: web::Data<ServerState>,
    input: web::Json<DownloadQuery>,
) -> impl Responder {
    let url = input.download_url.to_owned();
    let url_type = match check_url(&url) {
        Some(u) => u,
        None => return "Incorrect URL",
    };

    let row_id = match insert_new_download(state.db_conn.clone(), &url) {
        Ok(r) => r,
        Err(why) => {
            error!("{}", why);
            return "Error starting the download";
        }
    };

    let db_conn = state.db_conn.clone();
    tokio::spawn(async move {
        let download_result = match url_type {
            url::UrlType::WeTransfer => downloader::wetransfer::download(url).await,
            url::UrlType::YouTube => downloader::youtube::download(url).await,
        };
        debug!("{:?}", download_result);
        update_download(db_conn, row_id, download_result).unwrap_or_else(|e| error!("{}", e));
    });
    "Download has started"
}

#[get("/data")]
async fn get_data(state: web::Data<ServerState>) -> impl Responder {
    let result = select_data(state.db_conn.clone()).unwrap_or_default();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_owned()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_LOG", "debug");
    #[cfg(not(debug_assertions))]
    std::env::set_var("RUST_LOG", "info");

    pretty_env_logger::init();

    let db_conn = prepare_connection(DB_FILE);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            // .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(web::Data::new(ServerState {
                db_conn: db_conn.clone(),
            }))
            .service(health_check)
            .service(download)
            .service(get_data)
            .service(actix_files::Files::new("/", FILES_DIR).index_file("index.html"))
    })
    .bind(("0.0.0.0", 8055))?
    .run()
    .await
}
