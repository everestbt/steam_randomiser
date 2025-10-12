use rusqlite::{params, Connection, Result};
use chrono::{Local};

use db_lib::db_manager;

struct RequestCount {
    date: String,
    count: i32,
}

pub fn increment() -> Result<bool> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;
    
    let mut stmt = conn.prepare("SELECT date, count FROM steam_request_count")?;
    let mut result = stmt.query_map([], |row| {
        Ok(RequestCount {
            date: row.get(0)?,
            count: row.get(1)?,
        })
    })?;

    let row = result.next();
    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let count;
    if row.is_none() {
        conn.execute(
            "INSERT INTO steam_request_count (date, count) VALUES (?1,?2)",
            params![today, 1],
        )?;
        count = 1;
    }
    else {
        let current_count = row.unwrap().unwrap();        
        if current_count.date == today {
            count = current_count.count + 1;
        }
        else {
            count = 1;
        }
        // Clear the table and then add in the latest value
        conn.execute(
            "DELETE FROM steam_request_count",
            [], // No parameters needed
        )?;
        conn.execute(
                "INSERT INTO steam_request_count (date, count) VALUES (?1,?2)",
                params![today, count],
            )?;
    }
    
    // Check that the value is less than or equal to the set limit of 10000
    Ok(count <= 10000)
}

pub fn get_count() -> Result<i32> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let mut stmt = conn.prepare("SELECT date, count FROM steam_request_count WHERE date = ?1")?;
    let mut result = stmt.query_map([today], |row| {
        Ok(RequestCount {
            date: row.get(0)?,
            count: row.get(1)?,
        })
    })?;

    let row = result.next();
    let count;
    if row.is_none() {
        count = 0;
    }
    else {
        count = row.unwrap().unwrap().count;
    }

    Ok(count)
}

fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_request_count (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            count INTEGER NOT NULL
        )",
        [], // No parameters needed
    )?;

    Ok(())
}