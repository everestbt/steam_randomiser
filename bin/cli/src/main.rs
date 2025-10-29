use api::{achievement_fetch, game_fetch};
use db::{key_store, steam_id_store, achievement_store, excluded_achievement_store, request_store};

use std::collections::{HashMap};
use rand::prelude::*;
use clap::Parser;

// Command line arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Your Steam API key
    #[arg(short, long)]
    key: Option<String>,

    /// Your Steam ID
    #[arg(short, long)]
    id: Option<String>,

    /// Return a random achievement
    #[arg(short, long)]
    random_achievement: bool,

    /// Return a list of the current goals
    #[arg(short, long)]
    goals: bool,

    /// Exclude an achievement by its id in the goals list
    #[arg(short, long)]
    exclude_achievement: Option<i32>,

    /// Return a list of completed games
    #[arg(short, long)]
    completed_games: bool,

    /// Game name used for certain commands
    #[arg(short, long)]
    game_name: Option<String>,

    /// Show debug level information
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Args::parse();

    let key: String;
    if args.key.is_some() {
        key = args.key.unwrap();
        key_store::save_key(&key).expect("Failed to save the key");
        println!("Saved your key, no need to use --key each time now. You can replace it by using --key again.")
    }
    else {
        key = key_store::get_key().expect("Failed to load a key, use --key first");
    }

    let steam_id: String;
    if args.id.is_some() {
        steam_id = args.id.unwrap();
        steam_id_store::save_id(&steam_id).expect("Failed to save the key");
        println!("Saved your steam id, no need to use --id each time now. You can replace it by using --id again.")
    }
    else {
        steam_id = steam_id_store::get_id().expect("Failed to load a key, use --id first");
    }

    

    if args.random_achievement {
        let game_name: String = args.game_name.expect("Must pass in a game-name");
        println!("Searching for {:#?}", game_name);
        // Fetch games
        let owned_games: Vec<game_fetch::Game> = game_fetch::get_owned_games(&key, &steam_id).await;

        // Search for the game title
        let game_name_lowercase: String = game_name.to_lowercase();
        let filter_to_game: Vec<&game_fetch::Game> = owned_games
            .iter()
            .filter(|a| a.name.to_lowercase().contains(&game_name_lowercase))
            .collect();

        if filter_to_game.is_empty() {
            println!("Failed to find that game");
        } else if filter_to_game.len() > 1 {
            println!("Result is ambiguous");
            for ele in filter_to_game.iter() {
                println!("{:#?}!", ele.name);
            }
            println!("TAKING THE FIRST ONE:");
        }

        let game = filter_to_game.first().unwrap();

        println!("Found the game {:#?}!", game.name);

        // Get the achievements for a specific game
        let achievements = achievement_fetch::get_player_achievements(&key, &steam_id, &game.appid).await;
        if achievements.is_none() {
            println!("{game} doesn't have any achievements", game = game_name);
        }
        else {
            let player_achievements: Vec<achievement_fetch::PlayerAchievement> = achievements.unwrap().achievements;

            println!("Found the achievements!");

            // Get details of the achievements
            let achievements: Vec<achievement_fetch::GameAchievement> = achievement_fetch::get_game_achievements(&key, &game.appid).await;

            println!("Got the achievement details!");

            // Load currently listed achievements
            let current_goals_for_app: Vec<achievement_store::Achievement> = achievement_store::get_achievements_for_app(&game.appid).expect("Failed to load current goals");

            // Load excluded achievement
            let excluded_achievement_for_app: Vec<excluded_achievement_store::ExcludedAchievement> = excluded_achievement_store::get_excluded_achievements_for_app(&game.appid).expect("Failed to load excluded achievements");

            // Randomly select achievement from game
            let filter_to_unachieved: Vec<&achievement_fetch::PlayerAchievement> = player_achievements
                .iter()
                .filter(|a| a.achieved == 0) // Filter out achieved
                .filter(|a| !current_goals_for_app.iter().any(|x| x.achievement_name == a.apiname)) // Filter out already in goals
                .filter(|a| !excluded_achievement_for_app.iter().any(|x| x.achievement_name == a.apiname)) // Filter out any excluded achievements
                .collect();

            println!(
                "You have {:#?} achievements left in this game",
                filter_to_unachieved.len() + current_goals_for_app.len()
            );

            // Check there is something still in it
            if filter_to_unachieved.is_empty() {
                println!("Nothing left to add to goals!");
            }
            else {
                let mut rng = rand::rng();
                let random_achievement = filter_to_unachieved.choose(&mut rng).unwrap();
                let selected_achievement_desc: Vec<&achievement_fetch::GameAchievement> = achievements
                    .iter()
                    .filter(|a| a.name == random_achievement.apiname)
                    .collect();
                println!("And your selected achievement is:");
                let a = selected_achievement_desc.first().unwrap();

                println!(
                    "{achievement} : {description}",
                    achievement = a.display_name,
                    description = a
                        .description
                        .clone()
                        .unwrap_or("no description".to_string())
                );
                // Save the achievement
                achievement_store::save_achievement(&a.name, &game.appid).expect("Failed to save achievement");
                println!("Saved the achievement!");
            }
        }
    }
    else if args.goals {
        let mut achievements: Vec<achievement_store::Achievement> = achievement_store::get_achievements().expect("Failed to load achievements");
        achievements.sort_by(|a, b| i32::cmp(&a.app_id,&b.app_id));
        let mut app_player_achievement_map: HashMap<i32, achievement_fetch::PlayerAchievements> = HashMap::new();
        let mut app_game_achievement_map: HashMap<i32, Vec<achievement_fetch::GameAchievement>> = HashMap::new();
        for a in achievements {
            // Check if the app is already loaded (PlayerAchievements)
            let player_achievements = app_player_achievement_map.get(&a.app_id);
            let loaded_player: &achievement_fetch::PlayerAchievements;
            if player_achievements.is_none() {
                let player = achievement_fetch::get_player_achievements(&key, &steam_id, &a.app_id).await.expect("Somehow a game with no achievements has ended up with one?!?");
                app_player_achievement_map.insert(a.app_id, player);
                loaded_player = app_player_achievement_map.get(&a.app_id).unwrap();
            }
            else {
                loaded_player = player_achievements.unwrap();
            }
            // If a game-name is not contained, then filter it out
            if args.game_name.clone().is_some_and(|x| !loaded_player.game_name.to_lowercase().contains(&x.to_lowercase())) {
                continue;
            }
            // Check if the app is already loaded (GameAchievements)
            let game_achieveements = app_game_achievement_map.get(&a.app_id);
            let loaded_game_achievement: &Vec<achievement_fetch::GameAchievement>;
            if game_achieveements.is_none() {
                let game_achieve = achievement_fetch::get_game_achievements(&key, &a.app_id).await;
                app_game_achievement_map.insert(a.app_id, game_achieve);
                loaded_game_achievement = app_game_achievement_map.get(&a.app_id).unwrap();
            }
            else {
                loaded_game_achievement = game_achieveements.unwrap();
            }

            // Find the name of the achievement
            let found_achievement = loaded_game_achievement.iter().find(|ga| a.achievement_name == ga.name).unwrap();

            // Remove any that are already completed
            if loaded_player.achievements.iter().find(|x| x.apiname==a.achievement_name).unwrap().achieved == 1 {
                achievement_store::delete_achievement(&a.id).expect("Failed to delete achievement");
                println!("Well done! You completed {game} : {name}", name = found_achievement.display_name, game = loaded_player.game_name);
                continue;
            } 
            
            if found_achievement.description.is_none() {
                println!("{game} : {name} [{id}]", name = found_achievement.display_name, game = loaded_player.game_name, id = a.id);
            }
            else{
                println!("{game} : {name} - {description} [{id}]", name = found_achievement.display_name, game = loaded_player.game_name, description = found_achievement.description.clone().unwrap(), id = a.id);
            }
        }
    }
    else if args.exclude_achievement.is_some() {
        let achievement = achievement_store::get_achievement(&args.exclude_achievement.unwrap()).expect("Achievement no found");
        // First delete the achievement, if this is all that succeeds then it is at least off the list
        achievement_store::delete_achievement(&args.exclude_achievement.unwrap()).expect("Failed to delete achievement");
        // Add it to the list of excluded achievements
        excluded_achievement_store::save_excluded_achievement(&achievement.achievement_name, &achievement.app_id).expect("Failed to save the exclusion");
    }
    else if args.completed_games {
        // Get full game list
        let games: Vec<game_fetch::Game> = game_fetch::get_owned_games(&key, &steam_id).await;
        for game in games {
            // Skip the game if no playtime
            if game.playtime_forever == 0 {
                continue;
            }
            // Check for any excluded achievements
            let excluded_achievements: Vec<String> = excluded_achievement_store::get_excluded_achievements_for_app(&game.appid).expect("Failed to load excluded achievements").iter().map(|a| a.achievement_name.clone()).collect();
            // Get the achievements completed for that game
            let player_achievements = achievement_fetch::get_player_achievements(&key, &steam_id, &game.appid).await;
            if player_achievements.is_none() {
                continue;
            }
            let unachieved = player_achievements.unwrap().achievements.iter()
                .filter(|a| !excluded_achievements.contains(&a.apiname))
                .filter(|a| a.achieved==0)
                .count();
            if unachieved == 0 {
                println!("Completed game: {name}", name = game.name);
            }
        }
    }

    if args.debug {
        let request_count = request_store::get_count().expect("Failed to load request count");
        println!("Request count {count}", count = request_count);
    }
    Ok(())
}
