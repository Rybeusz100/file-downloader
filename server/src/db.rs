use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use rusqlite::{params, Connection, Result};
use serde::Serialize;

use crate::DownloadResult;

#[derive(Serialize, Debug)]
pub struct DbRow {
    pub id: u64,
    pub url: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<usize>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub status: String,
}

pub fn prepare_connection(path: &str) -> Arc<Mutex<Connection>> {
    let db_conn = Arc::new(Mutex::new(Connection::open(path).unwrap()));
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
    db_conn.lock().unwrap().execute(query, []).unwrap();
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
        "INSERT INTO downloads VALUES (NULL, ?1, NULL, NULL, datetime(), NULL, 'in progress')";
    db_conn.execute(query, [url])?;

    let inserted_row_id: u64 =
        db_conn.query_row("SELECT last_insert_rowid()", [], |row| row.get(0))?;
    Ok(inserted_row_id)
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
                SET file_name = ?1, file_size = ?2, end_time = datetime(), status = 'finished'
                WHERE id = ?3
                ";
            db_conn.execute(
                query,
                params![download_result.file_name, download_result.file_size, row_id],
            )?;
        }
        Err(_) => {
            let query = "
                UPDATE downloads
                SET end_time = datetime(), status = 'failed'
                WHERE id = ?1
                ";
            db_conn.execute(query, [row_id])?;
        }
    }
    Ok(())
}

pub fn select_data(db_conn: Arc<Mutex<Connection>>) -> Result<Vec<DbRow>, Box<dyn Error>> {
    let db_conn = match db_conn.lock() {
        Ok(c) => c,
        Err(_) => return Err("Error locking db_conn".into()),
    };
    let query = "SELECT * FROM downloads";
    let mut stmt = db_conn.prepare(query)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(DbRow {
                id: row.get(0)?,
                url: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                start_time: row.get(4)?,
                end_time: row.get(5)?,
                status: row.get(6)?,
            })
        })?
        .map(|r| r.unwrap())
        .collect();

    Ok(rows)
}
