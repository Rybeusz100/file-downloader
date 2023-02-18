use actix_web::{get, Responder};

#[get("/health")]
async fn health_check() -> impl Responder {
    "Healthy"
}
