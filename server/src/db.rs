use rusqlite::{params, Connection, Result};
use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use crate::models::{Download, DownloadResult, User};

pub fn prepare_connection(path: &str) -> Arc<Mutex<Connection>> {
    let db_conn = Arc::new(Mutex::new(Connection::open(path).unwrap()));
    let queries = [
        "
            CREATE TABLE IF NOT EXISTS downloads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL,
            file_name TEXT,
            file_size INTEGER,
            start_time DATETIME NOT NULL,
            end_time DATETIME,
            status TEXT CHECK(status IN ('in progress', 'finished', 'failed')) NOT NULL,
            user_id INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id));
        ",
        "
            CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            password TEXT NOT NULL);
        ",
        "
            UPDATE downloads
            SET status = 'failed'
            WHERE status = 'in progress';
        ",
    ];
    for query in queries {
        db_conn.lock().unwrap().execute(query, []).unwrap();
    }
    db_conn
}

pub fn insert_new_download(
    db_conn: Arc<Mutex<Connection>>,
    url: &str,
    user_id: u64,
) -> Result<u64> {
    let db_conn = db_conn.lock().unwrap();
    let query =
        "INSERT INTO downloads VALUES (NULL, ?1, NULL, NULL, datetime(), NULL, 'in progress', ?2)";
    db_conn.execute(query, params![url, user_id])?;

    let inserted_row_id: u64 =
        db_conn.query_row("SELECT last_insert_rowid()", [], |row| row.get(0))?;
    Ok(inserted_row_id)
}

pub fn update_download(
    db_conn: Arc<Mutex<Connection>>,
    row_id: u64,
    result: Result<DownloadResult, Box<dyn Error + Send + Sync>>,
) -> Result<()> {
    let db_conn = db_conn.lock().unwrap();
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

pub fn select_data(db_conn: Arc<Mutex<Connection>>, user_id: u64) -> Result<Vec<Download>> {
    let db_conn = db_conn.lock().unwrap();
    let query = "SELECT * FROM downloads WHERE user_id = ?1";
    let mut stmt = db_conn.prepare(query)?;
    let rows = stmt
        .query_map([user_id], |row| {
            Ok(Download {
                id: row.get(0)?,
                url: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                start_time: row.get(4)?,
                end_time: row.get(5)?,
                status: row.get(6)?,
                user_id: row.get(7)?,
            })
        })?
        .map(|r| r.unwrap())
        .collect();

    Ok(rows)
}

pub fn insert_new_user(db_conn: Arc<Mutex<Connection>>, name: &str, password: &str) -> Result<u64> {
    let db_conn = db_conn.lock().unwrap();

    let query = "INSERT INTO users VALUES (NULL, ?1, ?2)";
    db_conn.execute(query, [name, password])?;
    let inserted_row_id: u64 =
        db_conn.query_row("SELECT last_insert_rowid()", [], |row| row.get(0))?;
    Ok(inserted_row_id)
}

pub fn check_user_name_free(db_conn: Arc<Mutex<Connection>>, name: &str) -> Result<bool> {
    let db_conn = db_conn.lock().unwrap();

    let id: Result<u64> =
        db_conn.query_row("SELECT id FROM users WHERE name = ?1", [&name], |row| {
            row.get(0)
        });
    match id {
        Ok(_) => Ok(false),
        _ => Ok(true),
    }
}

pub fn get_user(
    db_conn: Arc<Mutex<Connection>>,
    id: Option<u64>,
    name: Option<String>,
) -> Result<Option<User>> {
    let db_conn = db_conn.lock().unwrap();

    let mut query = "SELECT id, name, password FROM users WHERE 1=1".to_owned();
    let mut params: Vec<&dyn rusqlite::types::ToSql> = Vec::new();

    if let Some(id) = id.as_ref() {
        query += " AND id = ?";
        params.push(id);
    }
    if let Some(name) = name.as_ref() {
        query += " AND name = ?";
        params.push(name);
    }

    let user: Result<User> = db_conn.query_row(&query, &params[..], |row| {
        Ok(User {
            id: row.get(0)?,
            name: row.get(1)?,
            password: row.get(2)?,
        })
    });

    Ok(user.ok())
}
