use crate::{
    db::{insert_new_download, select_data, update_download},
    downloader,
    url::{self, check_url},
    DownloadQuery, ServerState,
};
use actix_web::{get, post, web, HttpResponse, Responder};
use log::{debug, error};

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
