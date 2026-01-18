mod add_display_name_and_description_and_last_played_to_achievement_store;
mod drop_key_store;

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

    /// The migration to perform
    #[arg(short, long)]
    migration: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Args::parse();
    match args.migration.as_str() {
        "add_display_name_and_description_and_last_played_to_achievement_store" => {
            if args.id.is_none() || args.key.is_none() {
                println!("You need to suppy an API key and steam ID")
            }
            else {
                let result = add_display_name_and_description_and_last_played_to_achievement_store::run_migration(&args.key.unwrap(), &args.id.unwrap()).await;
                if result.is_err() {
                    println!("{error}", error = result.err().unwrap());
                }
                else {
                    println!("Success");
                }
            }
        },
        "drop_key_store" => {
            let result = drop_key_store::run_migration().await;
            if result.is_err() {
                println!("{error}", error = result.err().unwrap());
            }
            else {
                println!("Success");
            }
        },
        &_ => println!("Enter a migration to run")
    };
    Ok(())
}