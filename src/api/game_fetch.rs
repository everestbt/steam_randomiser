use serde::{Deserialize, Serialize};

// Owned Games Request
#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub appid: i32,
    pub name: String,
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

pub async fn get_owned_games(key : &str, steam_id : &str) -> Vec<Game> {
    // Fetch games for steamid
    let get_owned_games_request: String =
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?format=json&include_appinfo=true&include_played_free_games=true".to_owned() + "&key=" + key + "&steamid=" + steam_id;

    let req: Result<reqwest::Response, reqwest::Error> = reqwest::Client::new()
        .get(get_owned_games_request)
        .send()
        .await;

    match req.is_err() {
        true => panic!(),
        false => (),
    }
    let response: Result<SteamOwnedGamesResponse, reqwest::Error> = req.unwrap()
        .json()
        .await;

    match response.is_err() {
        true => panic!(),
        false => (),
    }

    response.unwrap().response.games
}