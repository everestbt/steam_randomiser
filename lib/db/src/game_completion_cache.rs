use rusqlite::{params, Connection, Result};

use db_lib::db_manager;

#[derive(Clone)]
pub struct GameCompletion {
    pub app_id: i32,
    pub complete: bool,
    pub last_played: i64,
    pub has_achievements: bool,
}

pub fn get_game_completion() -> Result<Vec<GameCompletion>> {
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT app_id, complete, last_played, has_achievements FROM steam_game_completion")?;
    let achieve_iter = stmt.query_map([], |row| {
        Ok(GameCompletion {
            app_id: row.get(0)?,
            complete: row.get(1)?,
            last_played: row.get(2)?,
            has_achievements: row.get(3)?,
        })
    })?;

    let mut vec : Vec<GameCompletion> = Vec::new();
    for d in achieve_iter {
        vec.push(d.unwrap());
    }
    Ok(vec)
}

pub fn save_game_completion(app_id: &i32, complete: bool, last_played: i64, has_achievements: bool) -> Result<()> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;
    
    // Add in the achievement
    conn.execute(
        "INSERT INTO steam_game_completion (app_id, complete, last_played, has_achievements) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(app_id) DO UPDATE SET complete=?5, last_played=?6, has_achievements=?7",
        params![app_id, complete, last_played, has_achievements, complete, last_played, has_achievements],
    )?;

    Ok(())
}

pub fn drop_table() -> Result<()> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = db_manager::get_connection();

        conn.execute(
        "DROP TABLE IF EXISTS steam_game_completion",
        [], // No parameters needed
    )?;

    Ok(())
}

fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_game_completion (
            app_id INTEGER PRIMARY KEY,
            complete BOOL NOT NULL,
            last_played INTEGER NOT NULL,
            has_achievements BOOL NOT NULL 
        )",
        [], // No parameters needed
    )?;

    Ok(())
}