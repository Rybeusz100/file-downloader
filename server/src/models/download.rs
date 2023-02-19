use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Download {
    pub id: u64,
    pub url: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<usize>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub status: Status,
    pub user_id: u64,
}

#[derive(Serialize)]
pub enum Status {
    #[serde(rename = "in progress")]
    InProgress,
    #[serde(rename = "finished")]
    Finished,
    #[serde(rename = "failed")]
    Failed,
}

impl FromSql for Status {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        match value.as_str()? {
            "in progress" => Ok(Status::InProgress),
            "finished" => Ok(Status::Finished),
            "failed" => Ok(Status::Failed),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DownloadRequest {
    pub download_url: String,
}

#[derive(Debug)]
pub struct DownloadResult {
    pub file_name: String,
    pub file_size: u64,
}
