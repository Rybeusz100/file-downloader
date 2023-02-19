use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use db::prepare_connection;
use hmac::Hmac;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use services::*;
use sha2::Sha256;
use std::sync::{Arc, Mutex};
use utils::generate_jwt_secret;
use validator::validator;

mod db;
mod downloader;
mod models;
mod services;
mod url;
mod utils;
mod validator;

const DB_FILE: &str = "./config/file-downloader.db";
const DOWNLOAD_DIR: &str = "./downloads/";
const FILES_DIR: &str = "../front/dist/";

struct AppState {
    db_conn: Arc<Mutex<Connection>>,
    jwt_secret: Hmac<Sha256>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenClaims {
    id: u64,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_LOG", "debug");
    #[cfg(not(debug_assertions))]
    std::env::set_var("RUST_LOG", "info");

    pretty_env_logger::init();

    let db_conn = prepare_connection(DB_FILE);
    let jwt_secret = generate_jwt_secret();

    HttpServer::new(move || {
        let bearer_middleware = HttpAuthentication::bearer(validator);
        App::new()
            .wrap(Cors::permissive())
            // .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(web::Data::new(AppState {
                db_conn: db_conn.clone(),
                jwt_secret: jwt_secret.clone(),
            }))
            .service(health_check)
            .service(create_user)
            .service(auth)
            .service(
                web::scope("/restricted")
                    .wrap(bearer_middleware)
                    .service(download)
                    .service(data),
            )
            .service(actix_files::Files::new("/", FILES_DIR).index_file("index.html"))
    })
    .bind(("0.0.0.0", 8055))?
    .run()
    .await
}
