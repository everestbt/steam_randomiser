use serde::{Deserialize, Serialize};

// Player Achievements Request
#[derive(Debug, Serialize, Deserialize)]
pub struct Achievement {
    pub apiname: String,
    pub achieved: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerAchievements {
    achievements: Vec<Achievement>,
    #[serde(rename = "gameName")]
    game_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerStatsResponse {
    playerstats: PlayerAchievements,
}

pub async fn get_player_achievements(key : &str, steam_id : &str, app_id : &str) -> Vec<Achievement> {
    let get_player_achievements_request: String = "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/?".to_owned()
        + "&key=" + key + "&steamid=" + steam_id
        + "&appid=" + app_id;

    let req: Result<reqwest::Response, reqwest::Error> = reqwest::Client::new()
        .get(get_player_achievements_request)
        .send()
        .await;

    match req.is_err() {
        true => panic!(),
        false => (),
    }

    let achievements:Result<PlayerStatsResponse, reqwest::Error>  = req.unwrap().json().await;

    match achievements.is_err() {
        true => panic!(),
        false => (),
    }

    achievements.unwrap().playerstats.achievements
}