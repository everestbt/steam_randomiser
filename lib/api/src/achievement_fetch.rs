use serde::{Deserialize, Serialize};
use db::request_store;

// Player Achievements Request
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerAchievement {
    pub apiname: String,
    pub achieved: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerAchievements {
    pub achievements: Vec<PlayerAchievement>,
    #[serde(rename = "gameName")]
    pub game_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerAchievementsInternal {
    achievements: Option<Vec<PlayerAchievement>>,
    #[serde(rename = "gameName")]
    game_name: Option<String>,
    success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerStatsResponse {
    playerstats: PlayerAchievementsInternal,
}

// Game Schema request
#[derive(Debug, Serialize, Deserialize)]
pub struct GameAchievement {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description: Option<String>,
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

pub async fn get_player_achievements(key : &str, steam_id : &str, app_id : &i32) -> Option<PlayerAchievements> {
    let get_player_achievements_request: String = "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/?".to_owned()
        + "&key=" + key + "&steamid=" + steam_id
        + "&appid=" + &app_id.to_string();

    if !request_store::increment().unwrap() {
        panic!("Hit request limit, wait until tomorrow");
    }
    let req: Result<reqwest::Response, reqwest::Error> = reqwest::Client::new()
        .get(get_player_achievements_request)
        .send()
        .await;

    match req.is_err() {
        true => panic!(),
        false => (),
    }

    let response:Result<PlayerStatsResponse, reqwest::Error>  = req.unwrap().json().await;

    match response.is_err() {
        true => panic!(),
        false => (),
    }

    let val = response.unwrap();
    // Success code can be true but no achievements present, typically for more modern games (Dota 2 is an example app_id 570)
    if val.playerstats.success && val.playerstats.achievements.is_some() {
        Option::Some(PlayerAchievements {
            achievements: val.playerstats.achievements.expect("Achievements should have been present..."),
            game_name: val.playerstats.game_name.expect("Game name should have been present..."),
        })
    }
    else {
        Option::None
    }
}

pub async fn get_game_achievements(key : &str, app_id : &i32) -> Vec<GameAchievement> {
    let get_schema_for_game_request: String =
        "https://api.steampowered.com/ISteamUserStats/GetSchemaForGame/v2/?key=".to_owned() + key + "&appid=" + &app_id.to_string();

    if !request_store::increment().unwrap() {
        panic!("Hit request limit, wait until tomorrow");
    }
    let req: Result<reqwest::Response, reqwest::Error> = reqwest::Client::new()
        .get(get_schema_for_game_request)
        .send()
        .await;

    match req.is_err() {
        true => panic!(),
        false => (),
    }

    let response:Result<GameSchemaResponse, reqwest::Error>  = req.unwrap().json().await;

    match response.is_err() {
        true => panic!(),
        false => (),
    }

    response.unwrap().game.available_game_stats.achievements
}