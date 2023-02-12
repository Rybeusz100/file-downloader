use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use db::prepare_connection;
use rusqlite::Connection;
use serde::Deserialize;
use services::*;
use std::sync::{Arc, Mutex};

mod db;
mod downloader;
mod services;
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
