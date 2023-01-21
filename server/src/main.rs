use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlite::Connection;
use std::sync::{Arc, Mutex};
use url::check_url;

mod url;
mod wetransfer;

#[derive(Deserialize, Debug)]
struct DownloadQuery {
    download_url: String,
}

struct ServerState {
    db_conn: Arc<Mutex<Connection>>,
}

#[derive(Serialize)]
struct DbRow {
    pub id: String,
    pub url: String,
    pub file_name: String,
    pub file_size: String,
    pub start_time: String,
    pub end_time: String,
    pub status: String,
}

#[derive(Serialize)]
struct DbSelectResult(Vec<DbRow>);

#[post("/download")]
async fn download(
    state: web::Data<ServerState>,
    input: web::Json<DownloadQuery>,
) -> impl Responder {
    let url = input.download_url.to_owned();
    let _url_type = match check_url(&url) {
        Some(u) => u,
        None => return "Incorrect URL",
    };

    let mut row_id = 0u32;
    {
        let conn = state.db_conn.lock().unwrap();
        let query =
            "INSERT INTO downloads VALUES (NULL, ?, NULL, NULL, datetime(), NULL, 'in progress')";
        let mut prepared = conn.prepare(query).unwrap();
        prepared.bind((1, url.as_str())).unwrap();
        prepared.next().unwrap();

        let query = "SELECT id FROM downloads ORDER BY id DESC limit 1";
        conn.iterate(query, |pairs| {
            for &(name, value) in pairs.iter() {
                if name == "id" {
                    row_id = value.unwrap().parse::<u32>().unwrap();
                }
            }
            true
        })
        .unwrap();
    }

    let db_conn = state.db_conn.clone();
    tokio::spawn(async move {
        match wetransfer::download(url).await {
            Ok(result) => {
                let query = "
                UPDATE downloads
                SET file_name = ?, file_size = ?, end_time = datetime(), status = 'finished'
                WHERE id = ?
                ";
                let conn = db_conn.lock().unwrap();
                let mut prepared = conn.prepare(query).unwrap();
                prepared.bind((1, result.file_name.as_str())).unwrap();
                prepared
                    .bind((2, result.file_size.to_string().as_str()))
                    .unwrap();
                prepared.bind((3, row_id.to_string().as_str())).unwrap();
                prepared.next().unwrap();
            }
            Err(_) => {
                let conn = db_conn.lock().unwrap();
                let query = "
                UPDATE downloads
                SET end_time = datetime(), status = 'failed'
                WHERE id = ?
                ";
                let mut prepared = conn.prepare(query).unwrap();
                prepared.bind((1, row_id.to_string().as_str())).unwrap();
                prepared.next().unwrap();
            }
        };
    });
    "Download has started"
}

#[get("/data")]
async fn get_data(state: web::Data<ServerState>) -> impl Responder {
    let conn = state.db_conn.lock().unwrap();
    let query = "SELECT * FROM downloads";
    let mut result = DbSelectResult(Vec::new());
    conn.iterate(query, |pairs| {
        let mut row = DbRow {
            id: String::new(),
            url: String::new(),
            file_name: String::new(),
            file_size: String::new(),
            start_time: String::new(),
            end_time: String::new(),
            status: String::new(),
        };
        for &(name, value) in pairs.iter() {
            match name {
                "id" => row.id = value.unwrap_or("-").to_owned(),
                "url" => row.url = value.unwrap_or("-").to_owned(),
                "file_name" => row.file_name = value.unwrap_or("-").to_owned(),
                "file_size" => row.file_size = value.unwrap_or("-").to_owned(),
                "start_time" => row.start_time = value.unwrap_or("-").to_owned(),
                "end_time" => row.end_time = value.unwrap_or("-").to_owned(),
                "status" => row.status = value.unwrap_or("-").to_owned(),
                _ => (),
            }
        }
        result.0.push(row);
        true
    })
    .unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&result).unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_LOG", "debug");
    #[cfg(not(debug_assertions))]
    std::env::set_var("RUST_LOG", "warn");

    pretty_env_logger::init();

    #[cfg(debug_assertions)]
    let files_dir = "../front/dist/";
    #[cfg(not(debug_assertions))]
    let files_dir = "./front/";

    let db_conn = Arc::new(Mutex::new(
        sqlite::open("./config/file-downloader.db").unwrap(),
    ));
    {
        let query = "
        CREATE TABLE IF NOT EXISTS downloads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL,
            file_name TEXT,
            file_size INTEGER,
            start_time DATETIME NOT NULL,
            end_time DATETIME,
            status TEXT CHECK(status IN ('in progress', 'finished', 'failed')) NOT NULL
        );";
        db_conn.lock().unwrap().execute(query).unwrap();
    }

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(ServerState {
                db_conn: db_conn.clone(),
            }))
            .service(get_data)
            .service(download)
            .service(actix_files::Files::new("/", files_dir).index_file("index.html"))
    })
    .bind(("0.0.0.0", 8055))?
    .run()
    .await
}
