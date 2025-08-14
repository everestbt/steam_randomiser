use rusqlite::{params, Connection, Result}; // For database operations and result handling

use steam_randomiser::db_lib::db_manager;

pub struct Achievement {
    pub id: i32,
    pub achievement_name: String,
    pub app_id: i32,
}

pub fn get_achievements() -> Result<Vec<Achievement>> {
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT id, achievement_name, app_id FROM steam_achievements")?;
    let achieve_iter = stmt.query_map([], |row| {
        Ok(Achievement {
            id: row.get(0)?,
            achievement_name: row.get(1)?,
            app_id: row.get(2)?,
        })
    })?;

    let mut achievement_vec : Vec<Achievement> = Vec::new();
    for d in achieve_iter {
        achievement_vec.push(d.unwrap());
    }
    Ok(achievement_vec)
}

pub fn get_achievements_for_app(app_id: &i32) -> Result<Vec<Achievement>> {
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;

    let mut stmt = conn.prepare("SELECT id, achievement_name, app_id FROM steam_achievements WHERE app_id = ?1")?;
    let achieve_iter = stmt.query_map([app_id], |row| {
        Ok(Achievement {
            id: row.get(0)?,
            achievement_name: row.get(1)?,
            app_id: row.get(2)?,
        })
    })?;

    let mut achievement_vec : Vec<Achievement> = Vec::new();
    for d in achieve_iter {
        achievement_vec.push(d.unwrap());
    }
    Ok(achievement_vec)
}

pub fn save_achievement(achievement_name: &String, app_id: &i32) -> Result<()> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = db_manager::get_connection();
    create_table(&conn)?;
    
    // Add in the achievement
    conn.execute(
        "INSERT INTO steam_achievements (achievement_name, app_id) VALUES (?1, ?2)",
        params![achievement_name, app_id],
    )?;

    Ok(())
}

pub fn delete_achievement(id: &i32) -> Result<()> {
    // Connect to SQLite database (creates the file if it doesn't exist)
    let conn: Connection = db_manager::get_connection();
    
    // Add in the achievement
    conn.execute(
        "DELETE FROM steam_achievements WHERE id = ?1",
        params![id],
    )?;

    Ok(())
}

fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_achievements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            achievement_name TEXT NOT NULL,
            app_id INTEGER NOT NULL
        )",
        [], // No parameters needed
    )?;

    Ok(())
}