use rusqlite::{Connection, params};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::{collections::HashMap};
use serde::{Serialize};

#[allow(dead_code)]
pub(crate) struct GodataDatabaseError {
    pub(crate) msg: String
}

pub(crate) fn table_exists(connection: Pool<SqliteConnectionManager>, table_name: &str) -> bool {
    let query = "SELECT name FROM sqlite_master WHERE type='table' AND name=?";
    let c = connection.get().unwrap();
    let mut stmt = c.prepare(&query).unwrap();
    let mut rows = stmt.query(params![table_name]).unwrap();
    let mut count = 0;
    while let Some(_a) = rows.next().unwrap() {
        count += 1;
    }
    count == 1
}

pub(crate) fn create_kv_table(connection: Pool<SqliteConnectionManager>, table_name: &str) -> Result<(), rusqlite::Error> {
    let query = &format!("CREATE TABLE \"{}\" (key STRING PRIMARY KEY, value STRING)", table_name);
    let c = connection.get().unwrap();
    let mut stmt = c.prepare(&query).unwrap();
    stmt.execute(params![]).unwrap();
    Ok(())
}

pub(crate) fn delete_kv_table(connection: Pool<SqliteConnectionManager>, table_name: &str) -> Result<(), rusqlite::Error> {
    let query = &format!("DROP TABLE \"{}\"", table_name);
    let c = connection.get().unwrap();
    let mut stmt = c.prepare(&query).unwrap();
    stmt.execute(params![]).unwrap();
    Ok(())
}

pub(crate) fn add_to_table(connection: Pool<SqliteConnectionManager>, table_name: &str, key: &str, value: &impl Serialize) -> Result<(), rusqlite::Error> {
    let query = &format!("INSERT INTO \"{}\" (key, value) VALUES (?, ?)", table_name);
    let c = connection.get().unwrap();
    let mut stmt = c.prepare(&query).unwrap();
    stmt.execute(params![key, serde_json::to_string(value).unwrap()]).unwrap();
    Ok(())
}

pub(crate) fn update_record(connection: Pool<SqliteConnectionManager>, table_name: &str, key: &str, value: &impl Serialize) -> Result<(), rusqlite::Error> {
    let query = &format!("UPDATE \"{}\" SET value=? WHERE key=?", table_name);
    let c = connection.get().unwrap();
    let mut stmt = c.prepare(&query).unwrap();
    stmt.execute(params![serde_json::to_string(value).unwrap(), key]).unwrap();
    Ok(())
}

pub(crate) fn list_tables(connection: Pool<SqliteConnectionManager>) -> Vec<String> {
    let query = "SELECT name FROM sqlite_master WHERE type='table'";
    let c = connection.get().unwrap();
    let mut stmt = c.prepare(&query).unwrap();
    let mut rows = stmt.query(params![]).unwrap();
    let mut tables = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        tables.push(row.get(0).unwrap());        
    }
    tables
}

pub(crate) fn get_record_from_table(connection: Pool<SqliteConnectionManager>, table_name: &str, key: &str) -> Option<String> {
    let query = &format!("SELECT * FROM \"{}\" WHERE key=?", table_name);
    let c = connection.get().unwrap();
    let mut stmt = c.prepare(&query).unwrap();
        
    let mut rows = stmt.query(params![key]).unwrap();
    let mut value = String::new();
    while let Some(row) = rows.next().unwrap() {
        value = row.get(1).unwrap();
    }
    if value.len() > 0 {
        Some(value)
    } else {
        None
    }
}

pub(crate) fn get_keys(connection: Pool<SqliteConnectionManager>, table_name: &str) -> Vec<String> {
    let query = &format!("SELECT key FROM \"{}\"", table_name);
    let c = connection.get().unwrap();
    let mut stmt = c.prepare(&query).unwrap();

    let mut rows = stmt.query(params![]).unwrap();
    let mut keys = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        keys.push(row.get(0).unwrap());
    }
    keys
}

pub(crate) fn remove(connection: Pool<SqliteConnectionManager>, table_name: &str, key: &str) -> Result<(), rusqlite::Error> {
    let query = &format!("DELETE FROM \"{}\" WHERE key=?", table_name);
    let c = connection.get().unwrap();
    let mut stmt = c.prepare(&query).unwrap();
    stmt.execute(params![key]).unwrap();
    Ok(())
}

pub(crate) fn n_records(connection: Pool<SqliteConnectionManager>, table_name: &str) -> Result<usize, rusqlite::Error> {
    let query = &format!("SELECT COUNT(*) FROM \"{}\"", table_name);
    let c = connection.get().unwrap();
    let mut stmt = c.prepare(&query)?;


    let mut rows = stmt.query(params![]).unwrap();
    let mut count = 0;
    while let Some(row) = rows.next().unwrap() {
        count = row.get(0).unwrap();
    }
    Ok(count)
}

pub(crate) fn get_all_records(connection: Pool<SqliteConnectionManager>, table_name: &str) -> Result<HashMap<String, String>, rusqlite::Error> {
    let query = &format!("SELECT * FROM \"{}\"", table_name);
    let c = connection.get().unwrap();
    let mut stmt = c.prepare(&query)?;
    let mut rows = stmt.query(params![]).unwrap();
    let mut records = HashMap::new();
    while let Some(row) = rows.next().unwrap() {
        let key = row.get(0).unwrap();
        let value = row.get(1).unwrap();
        records.insert(key, value);
    }
    Ok(records)
}