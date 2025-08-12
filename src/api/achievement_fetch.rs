use serde::{Deserialize, Serialize};

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
struct PlayerStatsResponse {
    playerstats: PlayerAchievements,
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

pub async fn get_player_achievements(key : &str, steam_id : &str, app_id : &i32) -> PlayerAchievements {
    let get_player_achievements_request: String = "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/?".to_owned()
        + "&key=" + key + "&steamid=" + steam_id
        + "&appid=" + &app_id.to_string();

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

    response.unwrap().playerstats
}

pub async fn get_game_achievements(key : &str, app_id : &i32) -> Vec<GameAchievement> {
    let get_schema_for_game_request: String =
        "https://api.steampowered.com/ISteamUserStats/GetSchemaForGame/v2/?key=".to_owned() + key + "&appid=" + &app_id.to_string();

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