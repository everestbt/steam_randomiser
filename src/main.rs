mod db;

use steam_randomiser::achievement_fetch;
use steam_randomiser::game_fetch;
use db::{key_store, steam_id_store};

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

    /// Game name used for certain commands
    #[arg(short, long)]
    game_name: String,
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

    let game_name: String = args.game_name;
    println!("Searching for {:#?}", game_name);

    if args.random_achievement {
        // Fetch games
        let owned_games: Vec<game_fetch::Game> = game_fetch::get_owned_games(&key, &steam_id).await;

        // Search for the game title
        let game_name_lowercase: String = game_name.to_lowercase();
        let filter_to_game: Vec<&game_fetch::Game> = owned_games
            .iter()
            .filter(|a| a.name.to_lowercase().contains(&game_name_lowercase))
            .collect();

        if filter_to_game.len() == 0 {
            println!("Failed to find that game");
        } else if filter_to_game.len() > 1 {
            println!("Result is ambiguous");
            for ele in filter_to_game.iter() {
                println!("{:#?}!", ele.name);
            }
            println!("TAKING THE FIRST ONE:");
        }

        let game = filter_to_game.get(0).unwrap();

        let app_id: &str = &game.appid.to_string();

        println!("Found the game {:#?}!", game.name);

        // Get the achievements for a specific game
        let player_achievements: Vec<achievement_fetch::PlayerAchievement> = achievement_fetch::get_player_achievements(&key, &steam_id, app_id).await;

        println!("Found the achievements!");

        // Get details of the achievements
        let achievements: Vec<achievement_fetch::GameAchievement> = achievement_fetch::get_game_achievements(&key, app_id).await;

        println!("Got the achievement details!");

        // Randomly select achievement from game
        let filter_to_unachieved: Vec<&achievement_fetch::PlayerAchievement> = player_achievements
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
            let selected_achievement_desc: Vec<&achievement_fetch::GameAchievement> = achievements
                .iter()
                .filter(|a| a.name == random_achievement.unwrap().apiname)
                .collect();
            println!("And your selected achievement is:");
            let a = selected_achievement_desc.get(0).unwrap();

            println!(
                "{achievement} : {description}",
                achievement = a.display_name,
                description = a
                    .description
                    .clone()
                    .unwrap_or("no description".to_string())
            );
        }
    }
    Ok(())
}
