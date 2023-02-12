use crate::{
    db::{
        check_user_name_free, insert_new_download, insert_new_user, select_data, update_download,
    },
    downloader,
    url::{self, check_url},
    CreateUserQuery, DownloadQuery, ServerState,
};
use actix_web::{get, post, web, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
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

#[post("/create_user")]
async fn create_user(
    state: web::Data<ServerState>,
    input: web::Json<CreateUserQuery>,
) -> impl Responder {
    let input: CreateUserQuery = input.into_inner();

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(input.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    match check_user_name_free(state.db_conn.clone(), &input.name) {
        Ok(false) => return format!("User with name {} already exists", input.name),
        Err(why) => {
            error!("{}", why);
            return "Error creating the user".to_owned();
        }
        Ok(true) => (),
    }

    match insert_new_user(state.db_conn.clone(), &input.name, &password_hash) {
        Ok(id) => id.to_string(),
        Err(why) => {
            error!("{}", why);
            "Error creating the user".to_owned()
        }
    }
}
