use actix_web::{
    get, post,
    web::{self, ReqData},
    HttpResponse, Responder,
};
use log::{debug, error};

use crate::{
    db::{get_user, insert_new_download, select_data, update_download},
    downloader,
    url::{self, check_url},
    AppState, DownloadQuery, TokenClaims,
};

#[post("/download")]
async fn download(
    req_user: Option<ReqData<TokenClaims>>,
    state: web::Data<AppState>,
    input: web::Json<DownloadQuery>,
) -> impl Responder {
    let user_id = match req_user {
        None => return "User validated but no ID???",
        Some(user) => user.id,
    };
    let url = input.download_url.to_owned();
    let url_type = match check_url(&url) {
        Some(u) => u,
        None => return "Incorrect URL",
    };

    let row_id = match insert_new_download(state.db_conn.clone(), &url, user_id) {
        Ok(r) => r,
        Err(why) => {
            error!("{}", why);
            return "Error starting the download";
        }
    };

    let db_conn = state.db_conn.clone();

    let user = match get_user(db_conn.clone(), Some(user_id), None) {
        Ok(Some(u)) => u,
        _ => {
            return "Error preparing user data";
        }
    };

    tokio::spawn(async move {
        let download_result = match url_type {
            url::UrlType::WeTransfer => downloader::wetransfer::download(url, &user.name).await,
            url::UrlType::YouTube => downloader::youtube::download(url, &user.name).await,
        };
        debug!("{:?}", download_result);
        update_download(db_conn, row_id, download_result).unwrap_or_else(|e| error!("{}", e));
    });
    "Download has started"
}

#[get("/data")]
async fn data(
    req_user: Option<ReqData<TokenClaims>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let result = match req_user {
        None => Vec::new(),
        Some(user) => select_data(state.db_conn.clone(), user.id).unwrap_or_default(),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_owned()))
}
