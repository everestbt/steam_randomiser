use api::{achievement_fetch::{self, GameAchievement}, game_fetch};
use db::{achievement_store, excluded_achievement_store};

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