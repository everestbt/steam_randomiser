use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::env;

// Owned Games Request
#[derive(Debug, Serialize, Deserialize)]
struct Game {
    appid: i32,
    name: String,
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

// Player Achievements Request
#[derive(Debug, Serialize, Deserialize)]
struct Achievement {
    apiname: String,
    achieved: i32,
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

// Game Schema request
#[derive(Debug, Serialize, Deserialize)]
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

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args: Vec<String> = env::args().collect();

    let key: &str = &args[1];
    let key_add: &str = "&key=";

    let steam_id: &str = &args[2];
    let steam_id_add: &str = "&steamid=";

    let app_id_add: &str = "&appid=";

    let mut mut_game_name = String::new() + &args[3];
    for i in 4..args.len() {
        mut_game_name.push_str(" ");
        mut_game_name.push_str(&args[i].to_owned());
    }
    let game_name: &str = mut_game_name.as_str();
    println!("Searching for {:#?}", game_name);

    // Fetch games for steamid
    let base_owned_games_request: String = "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?format=json&include_appinfo=true&include_played_free_games=true".to_owned();
    let get_owned_games_request: String =
        base_owned_games_request + key_add + key + steam_id_add + steam_id;

    let owned_games: SteamOwnedGamesResponse = reqwest::Client::new()
        .get(get_owned_games_request)
        .send()
        .await?
        .json()
        .await?;

    let filter_to_game: Vec<&Game> = owned_games
        .response
        .games
        .iter()
        .filter(|a| a.name == game_name)
        .collect();

    if filter_to_game.len() != 1 {
        println!("Failed to find that game");
    }

    let app_id: &str = &filter_to_game.get(0).unwrap().appid.to_string();

    println!("Found the game!");

    // Get the achievements for a specific game
    let base_get_player_achievements_request: String =
        "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/?".to_owned();
    let get_player_achievements_request: String = base_get_player_achievements_request
        + key_add
        + key
        + steam_id_add
        + steam_id
        + app_id_add
        + app_id;

    let player_achievements: PlayerStatsResponse = reqwest::Client::new()
        .get(get_player_achievements_request)
        .send()
        .await?
        .json()
        .await?;

    println!("Found the achievements!");

    // Get details of the achievements
    let base_get_schema_for_game_request: String =
        "https://api.steampowered.com/ISteamUserStats/GetSchemaForGame/v2/?key=".to_owned();
    let get_schema_for_game_request: String =
        base_get_schema_for_game_request + key + app_id_add + app_id;

    let achievements: GameSchemaResponse = reqwest::Client::new()
        .get(get_schema_for_game_request)
        .send()
        .await?
        .json()
        .await?;

    println!("Got the achievement details!");

    // Randomly select achievement from game
    let filter_to_unachieved: Vec<&Achievement> = player_achievements
        .playerstats
        .achievements
        .iter()
        .filter(|a| a.achieved == 0)
        .collect();

    println!(
        "You have {:#?} achievements left in this game",
        filter_to_unachieved.len()
    );

    // Check there is something still in it
    if filter_to_unachieved.len() == 0 {
        println!("Nothing left to achieve");
    }
    let mut rng = rand::rng();
    let random_achievement = filter_to_unachieved.choose(&mut rng);
    if random_achievement.is_none() {
        println!("Nothing left to achieve");
    } else {
        let selected_achievement_desc: Vec<&GameAchievement> = achievements
            .game
            .available_game_stats
            .achievements
            .iter()
            .filter(|a| a.name == random_achievement.unwrap().apiname)
            .collect();
        println!("And your selected achievement is:");
        let a = selected_achievement_desc.get(0).unwrap();
        println!("{:#?}", a.display_name);
        if a.description.is_some() {
            println!("{:#?}", a.description.clone().unwrap());
        }
    }
    Ok(())
}
