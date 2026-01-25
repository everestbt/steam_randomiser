use api::{achievement_fetch::{self, GameAchievement}, game_fetch};
use db::{achievement_store, excluded_achievement_store, game_completion_cache};

use std::{collections::HashMap};
use rand::prelude::*;

pub async fn get_random_achievement_for_game(key : &str, steam_id : &str, game: &game_fetch::Game) -> Option<GameAchievement> {
    // Get the achievements for a specific game
        let achievements = achievement_fetch::get_player_achievements(&key, &steam_id, &game.appid).await;
        if achievements.is_none() {
            return None;
        }
        else {
            let player_achievements: Vec<achievement_fetch::PlayerAchievement> = achievements.unwrap().achievements;
            // Get details of the achievements
            let achievements: Vec<achievement_fetch::GameAchievement> = achievement_fetch::get_game_achievements(&key, &game.appid).await;

            // Load currently listed achievements
            let current_goals_for_app: Vec<achievement_store::Achievement> = achievement_store::get_achievements_for_app(&game.appid).expect("Failed to load current goals");

            // Load excluded achievement
            let excluded_achievement_for_app: Vec<excluded_achievement_store::ExcludedAchievement> = excluded_achievement_store::get_excluded_achievements_for_app(&game.appid).expect("Failed to load excluded achievements");

            // Randomly select achievement from game
            let filter_to_unachieved: Vec<achievement_fetch::PlayerAchievement> = player_achievements
                .iter()
                .filter(|a| a.achieved == 0) // Filter out achieved
                .filter(|a| !current_goals_for_app.iter().any(|x| x.achievement_name == a.apiname)) // Filter out already in goals
                .filter(|a| !excluded_achievement_for_app.iter().any(|x| x.achievement_name == a.apiname)) // Filter out any excluded achievements
                .cloned()
                .collect();

            // Check there is something still in it
            if filter_to_unachieved.is_empty() {
                return None;
            }
            else {
                let mut rng = rand::rng();
                let random_achievement = filter_to_unachieved.choose(&mut rng).unwrap();
                return Some(achievements
                    .iter()
                    .find(|a| a.name == random_achievement.apiname).cloned().unwrap())
            }
        }
}

pub async fn refresh_game_completion_cache(key : &str, steam_id : &str, games: &Vec<game_fetch::Game>) {
    // Get cached completed games
    let completed_games_cache: HashMap<i32, game_completion_cache::GameCompletion> = game_completion_cache::get_game_completion().expect("Failed to load completed games").iter().map(|n| (n.app_id, n.clone())).collect();
    for game in games {
        // Skip the game if no playtime
        if game.playtime_forever == 0 {
            continue;
        }
        // Check if cached and not played since
        let cache_check = completed_games_cache.get(&game.appid);
        if cache_check.is_some_and(|c| c.last_played == game.last_played) {
            continue;
        }
        // Check for any excluded achievements
        let excluded_achievements: Vec<String> = excluded_achievement_store::get_excluded_achievements_for_app(&game.appid).expect("Failed to load excluded achievements").iter().map(|a| a.achievement_name.clone()).collect();
        // Get the achievements completed for that game
        let player_achievements = achievement_fetch::get_player_achievements(&key, &steam_id, &game.appid).await;
        if player_achievements.is_none() {
            // Game has no achievements, save it as completed
            game_completion_cache::save_game_completion(&game.appid, 100, game.last_played, false).expect("Failed to save game completion");
            continue;
        }
        let p = player_achievements.unwrap();
        let unachieved = p.achievements.iter()
            .filter(|a| !excluded_achievements.contains(&a.apiname))
            .filter(|a| a.achieved==0)
            .count();
        // Display if it is complete and save the current result
        if unachieved == 0 {
            game_completion_cache::save_game_completion(&game.appid, 100, game.last_played, true).expect("Failed to save game completion");
        }
        else {
            let progress : i8 = (100.0 * (1.0 -( (unachieved as f32) / (p.achievements.len() as f32)))) as i8;
            game_completion_cache::save_game_completion(&game.appid, progress, game.last_played, true).expect("Failed to save game completion");
        }
    }
}   