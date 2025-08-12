use rusqlite::{Connection};
use directories::{ProjectDirs};
use std::fs;

static DATABASE_NAME: &'static str = "steam_randomiser_database.db";

pub fn get_connection() -> Connection {
    let binding = ProjectDirs::from("com", "everest", "steam_randomiser")
        .expect("Failed to get project directories");
    let data_dir =  binding.data_local_dir();
    if !fs::exists(data_dir).expect("Failed to check for directory") {
        fs::create_dir(data_dir).expect("Failed to create directory");
    }
    let path = data_dir.join(DATABASE_NAME);
    let conn: Connection = Connection::open(path).expect("Failed to open a connection");
    return conn;
}