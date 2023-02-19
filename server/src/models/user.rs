use serde::{Deserialize, Serialize};

pub struct User {
    pub id: u64,
    pub name: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub password: String,
}
