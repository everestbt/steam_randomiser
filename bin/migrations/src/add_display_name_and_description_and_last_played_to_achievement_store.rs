use rusqlite::{params, Connection, Result};
use directories::{ProjectDirs};
use std::fs;
use serde::{Deserialize, Serialize};


struct Achievement {
    id: i32,
    achievement_name: String,
    app_id: i32,
}

pub async fn run_migration(key : &str, steam_id : &str) -> Result<String, String> {
    let conn = get_connection();
    // First create the table if it doesn't exist, this makes sure the migrations runs even if this is the first time running
    let create_original_table = conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_achievements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            achievement_name TEXT NOT NULL,
            app_id INTEGER NOT NULL
        )",
        [], // No parameters needed
    );
    if create_original_table.is_err() {
        return Err(create_original_table.err().unwrap().to_string());
    }

    // As we need to update data depending on values already present we will need a new table, create that if it doesn't exist
    let create_new_table = conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_achievements_v_2 (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            achievement_name TEXT NOT NULL,
            display_name TEXT NOT NULL,
            app_id INTEGER NOT NULL,
            description TEXT,
            last_played INTEGER NOT NULL
        )",
        [], // No parameters needed
    );
    if create_new_table.is_err() {
        return Err(create_new_table.err().unwrap().to_string());
    }
    println!("Created new table");

    // Read each value from the original table
    let prepare = conn.prepare("SELECT id, achievement_name, app_id FROM steam_achievements");
    if prepare.is_err() {
        return Err(prepare.err().unwrap().to_string());
    }
    let mut stmt = prepare.unwrap();
    let achieve_iter = stmt.query_map([], |row| {
        Ok(Achievement {
            id: row.get(0)?,
            achievement_name: row.get(1)?,
            app_id: row.get(2)?,
        })
    });
    if achieve_iter.is_err() {
        return Err(achieve_iter.err().unwrap().to_string());
    }
    let achieve_iter = achieve_iter.unwrap();
    let mut achievement_vec : Vec<Achievement> = Vec::new();
    for d in achieve_iter {
        achievement_vec.push(d.unwrap());
    }
    println!("Read all achievements");

    // For each value in the achievement vector call the API to get the values needed to populate the additional fields
    // First get all games
    let owned_games = get_owned_games(key, steam_id).await;
    for a in achievement_vec {
        // Get the game achievements
        let game_achievements = get_game_achievements(key, &a.app_id).await;
        // Find the specific achievement and game
        let achievement = game_achievements.iter().find(|ga| ga.name == a.achievement_name).unwrap();
        let game = owned_games.iter().find(|g| g.appid == a.app_id).unwrap();
        // Add in the achievement to the new table, ignoring any conflicts as it may be a restarted migration
        let write_result = conn.execute(
            "INSERT INTO steam_achievements_v_2 (id, achievement_name, display_name, app_id, description, last_played) VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(id) DO NOTHING",
            params![a.id, a.achievement_name, achievement.display_name, a.app_id, achievement.description, game.last_played],
        );
        if write_result.is_err() {
            return Err(write_result.err().unwrap().to_string());
        }   
        println!("Migrated [{id}, {name}, {display_name}, {app_id}]", id = a.id, name = a.achievement_name, display_name = achievement.display_name, app_id = a.app_id);
    }

    // Finally drop the old table
    let drop_original_table = conn.execute(
        "DROP TABLE IF EXISTS steam_achievements",
        [], // No parameters needed
    );
    if drop_original_table.is_err() {
        return Err(drop_original_table.err().unwrap().to_string());
    }   
    println!("Dropped old table");

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

// Owned Games Request
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Game {
    appid: i32,
    name: String,
    playtime_forever: i32, // This is the number of minutes played
    #[serde(rename = "rtime_last_played")]
    last_played: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct OwnedGames {
    game_count: i32,
    games: Vec<Game>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SteamOwnedGamesResponse {
    response: OwnedGames,
}

/// Fetch games for steamid
async fn get_owned_games(key : &str, steam_id : &str) -> Vec<Game> {

    let get_owned_games_request: String =
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?format=json&include_appinfo=true&include_played_free_games=true".to_owned() + "&key=" + key + "&steamid=" + steam_id;
    let req: Result<reqwest::Response, reqwest::Error> = reqwest::Client::new()
        .get(get_owned_games_request)
        .send()
        .await;

    if req.is_err() {
        panic!()
    }
    let response: Result<SteamOwnedGamesResponse, reqwest::Error> = req.unwrap()
        .json()
        .await;

    if response.is_err() {
        panic!()
    }

    response.unwrap().response.games
}

// Game Schema request
#[derive(Debug, Serialize, Deserialize, Clone)]
struct GameAchievement {
    name: String,
    #[serde(rename = "displayName")]
    display_name: String,
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameStats {
    achievements: Vec<GameAchievement>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AvailableGameStats {
    #[serde(rename = "availableGameStats")]
    available_game_stats: GameStats,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameSchemaResponse {
    game: AvailableGameStats,
}

async fn get_game_achievements(key : &str, app_id : &i32) -> Vec<GameAchievement> {
    let get_schema_for_game_request: String =
        "https://api.steampowered.com/ISteamUserStats/GetSchemaForGame/v2/?key=".to_owned() + key + "&appid=" + &app_id.to_string();
    let req: Result<reqwest::Response, reqwest::Error> = reqwest::Client::new()
        .get(get_schema_for_game_request)
        .send()
        .await;

    if req.is_err() {
        panic!()
    }

    let response:Result<GameSchemaResponse, reqwest::Error>  = req.unwrap().json().await;

    if response.is_err() {
        panic!()
    }

    response.unwrap().game.available_game_stats.achievements
}