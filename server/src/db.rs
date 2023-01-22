use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use serde::Serialize;
use sqlite::Connection;

use crate::DownloadResult;

#[derive(Serialize)]
pub struct DbRow {
    pub id: String,
    pub url: String,
    pub file_name: String,
    pub file_size: String,
    pub start_time: String,
    pub end_time: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct DbSelectResult(pub Vec<DbRow>);

pub fn prepare_connection(path: &str) -> Arc<Mutex<Connection>> {
    let db_conn = Arc::new(Mutex::new(sqlite::open(path).unwrap()));
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

    db_conn
}

pub fn insert_new_download(
    db_conn: Arc<Mutex<Connection>>,
    url: &str,
) -> Result<u64, Box<dyn Error>> {
    let db_conn = match db_conn.lock() {
        Ok(c) => c,
        Err(_) => return Err("Error locking db_conn".into()),
    };
    let query =
        "INSERT INTO downloads VALUES (NULL, ?, NULL, NULL, datetime(), NULL, 'in progress')";
    let mut prepared = db_conn.prepare(query)?;
    prepared.bind((1, url))?;
    prepared.next()?;

    let mut inserted_row_id = Err("Error finding inserted row".into());

    db_conn.iterate("SELECT last_insert_rowid()", |pairs| {
        match pairs.get(0) {
            Some((_, Some(opt))) => {
                inserted_row_id = opt.parse::<u64>().map_err(|e| e.into());
            }
            _ => inserted_row_id = Err("Error finding inserted row".into()),
        }
        true
    })?;

    inserted_row_id
}

pub fn update_download(
    db_conn: Arc<Mutex<Connection>>,
    row_id: u64,
    result: Result<DownloadResult, Box<dyn Error + Send + Sync>>,
) -> Result<(), Box<dyn Error>> {
    let db_conn = match db_conn.lock() {
        Ok(c) => c,
        Err(_) => return Err("Error locking db_conn".into()),
    };
    match result {
        Ok(download_result) => {
            let query = "
                UPDATE downloads
                SET file_name = ?, file_size = ?, end_time = datetime(), status = 'finished'
                WHERE id = ?
                ";
            let mut prepared = db_conn.prepare(query)?;
            prepared.bind((1, download_result.file_name.as_str()))?;
            prepared.bind((2, download_result.file_size.to_string().as_str()))?;
            prepared.bind((3, row_id.to_string().as_str()))?;
            prepared.next()?;
        }
        Err(_) => {
            let query = "
                UPDATE downloads
                SET end_time = datetime(), status = 'failed'
                WHERE id = ?
                ";
            let mut prepared = db_conn.prepare(query)?;
            prepared.bind((1, row_id.to_string().as_str()))?;
            prepared.next()?;
        }
    }

    Ok(())
}

pub fn select_data(db_conn: Arc<Mutex<Connection>>) -> Result<DbSelectResult, Box<dyn Error>> {
    let db_conn = match db_conn.lock() {
        Ok(c) => c,
        Err(_) => return Err("Error locking db_conn".into()),
    };
    let query = "SELECT * FROM downloads";
    let mut result = DbSelectResult(Vec::new());
    db_conn.iterate(query, |pairs| {
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
    })?;

    Ok(result)
}
