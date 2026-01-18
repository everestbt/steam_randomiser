use rusqlite::{Connection, Result};
use directories::{ProjectDirs};
use std::fs;

pub async fn run_migration() -> Result<String, String> {
    let conn: Connection = get_connection();
    let drop_table = conn.execute(
        "DROP TABLE IF EXISTS steam_key",
        [], // No parameters needed
    );
    if drop_table.is_err() {
        return Err(drop_table.err().unwrap().to_string());
    }
    
    Ok("Success".to_string())
}

fn get_connection() -> Connection {
    let binding = ProjectDirs::from("com", "everest", "steam_randomiser")
        .expect("Failed to get project directories");
    let data_dir =  binding.data_local_dir();
    if !fs::exists(data_dir).expect("Failed to check for directory") {
        fs::create_dir(data_dir).expect("Failed to create directory");
    }
    let path = data_dir.join("steam_randomiser_database.db");
    let conn: Connection = Connection::open(path).expect("Failed to open a connection");
    conn
}