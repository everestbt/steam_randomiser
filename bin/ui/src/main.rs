use api::game_fetch;
use db::{achievement_store, steam_id_store, game_completion_cache, excluded_achievement_store};
use goals_lib::goals;

use eframe::egui;
use std::{env, collections::HashSet, collections::HashMap};

#[derive(PartialEq)]
enum Sorting { Alphabetical, Progress }

fn main() -> eframe::Result {
    let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");

    let key_var = env::var("STEAM_API_KEY");
    if key_var.is_err() {
        panic!("You need to set the environment variable STEAM_API_KEY with your API key")
    }
    let key = key_var.unwrap();

    let steam_id = steam_id_store::get_id().expect("Failed to load steam-id, use the cli and supply a --id first");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([2000.0, 1200.0]),
        ..Default::default()
    };

    let mut game_list: Vec<game_fetch::Game> = runtime.block_on(game_fetch::get_owned_games(&key, &steam_id));
    let mut selected_game_app_id: HashSet<i32> = HashSet::new();
    let mut goals: Vec<achievement_store::Achievement> = get_goals();

    // Refresh the completed cache and fetch
    runtime.block_on(goals::refresh_game_completion_cache(&key, &steam_id, &game_list));
    let completed_games_cache: HashMap<i32, game_completion_cache::GameCompletion> = game_completion_cache::get_game_completion()
        .expect("Failed to load completed games")
        .iter()
        .map(|n| (n.app_id, n.clone()))
        .collect();

    game_list.sort_by(|a,b| completed_games_cache.get(&b.appid).map(|f| f.complete).unwrap_or(0).cmp(
        &completed_games_cache.get(&a.appid).map(|f| f.complete).unwrap_or(0)
    ));    

    // Actions
    let mut excluding_mode : bool = false;

    // Filters
    let mut filter_completed_game : bool = false;
    let mut filter_in_progress : bool = false;
    let mut filter_has_achievements : bool = true;
    let mut filter_perfect : bool = false;
    let mut filter_search : String = String::new();

    // Sorting
    let mut sorting = Sorting::Progress;

    // Refresh
    let mut refresh : bool = false;

    eframe::run_simple_native("Steam randomiser", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to Steam Randomiser");

            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {               
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    // List out all goals
                    ui.heading("Goals");
                    ui.add_space(5.0);
                    ui.checkbox(&mut excluding_mode, "Allow excluding");
                    ui.add_space(5.0);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            for g in &goals {
                                let result;
                                let game_name = game_list.iter().find(|game| game.appid == g.app_id).unwrap().name.clone();
                                if g.description.is_some() {
                                    result = format!("{} : {} : {}", game_name, g.display_name.clone(), g.description.clone().unwrap());
                                }
                                else {
                                    result = format!("{} : {}", game_name, g.display_name.clone());
                                }
                                if ui
                                    .add(egui::Label::new(result)
                                    .sense(egui::Sense::click()))
                                    .clicked() {
                                    if excluding_mode {
                                        achievement_store::delete_achievement(&g.id).expect("Failed to delete achievement");
                                        excluded_achievement_store::save_excluded_achievement(&g.achievement_name, &g.app_id).expect("Failed to save excluded achievement");
                                        refresh = true;
                                    }
                                };
                                ui.add_space(5.0);
                            }
                        });
                    });
                });  
                // List out goals for each selected items     
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.heading("Selected");
                    // Button to randomly select a game
                    if ui.add(egui::Button::new("Random game!")).clicked() {
                        let index = (rand::random::<f32>() * game_list.len() as f32).floor() as usize;
                        selected_game_app_id.insert(game_list[index].appid);
                    }
                    for s in game_list.iter().filter(|a: &&game_fetch::Game| selected_game_app_id.contains(&a.appid)) {
                        ui.add(egui::Label::new(s.name.clone()));
                        ui.add_space(5.0);
                        let progress = completed_games_cache.get(&s.appid).map(|c| c.complete).unwrap_or(0).to_string();
                        ui.add(egui::Label::new(format!("Progress [{}%]", progress)));
                        ui.add_space(5.0);
                        // Goals
                        ui.label("Goals:");
                        ui.add_space(5.0);
                        let game_goals = goals.iter().filter(|a| a.app_id == s.appid);
                        for a in game_goals {
                            let result;
                            if a.description.is_some() {
                                result = format!("{} : {}", a.display_name.clone(), a.description.clone().unwrap());
                            }
                            else {
                                result = format!("{}", a.display_name.clone());
                            }
                            ui.label(result);
                            ui.add_space(5.0);
                        }
                        if ui.add(egui::Button::new("Random Achievement")).clicked() {
                            let random_achievement = runtime.block_on(goals::get_random_achievement_for_game(&key, &steam_id, &s));
                            if random_achievement.is_some() {
                                let a = random_achievement.unwrap();
                                achievement_store::save_achievement(&a.name, &a.display_name, &a.description, &s.appid, &s.last_played).expect("Failed to save achievement");
                                refresh = true;
                            }
                        }
                        ui.add_space(5.0);
                    }   
                });
                // Display a list of every owned game that can then be selected/deselected
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.checkbox(&mut filter_in_progress, "In Progress");
                        ui.checkbox(&mut filter_completed_game, "Completed");
                        ui.checkbox(&mut filter_perfect, "Perfected");
                        ui.checkbox(&mut filter_has_achievements, "Has achievements");
                        ui.add(egui::TextEdit::singleline(&mut filter_search));

                        if ui.add(egui::RadioButton::new(sorting == Sorting::Alphabetical, "Alphabetical")).clicked() {
                            sorting = Sorting::Alphabetical;
                            game_list.sort_by(|a,b| a.name.cmp(&b.name));
                        }
                        if ui.add(egui::RadioButton::new(sorting == Sorting::Progress, "Progress")).clicked() {
                            sorting = Sorting::Progress;
                            game_list.sort_by(|a,b| completed_games_cache.get(&b.appid).map(|f| f.complete).unwrap_or(0).cmp(
                                &completed_games_cache.get(&a.appid).map(|f| f.complete).unwrap_or(0)
                            ));
                        }
                        for game in &mut game_list {
                            // check all filters
                            if filter_in_progress {
                                if completed_games_cache.get(&game.appid).map(|c| c.complete).unwrap_or(0) == 100 {
                                    continue;
                                }
                            }
                            if filter_completed_game {
                                if completed_games_cache.get(&game.appid).map(|c| c.complete).unwrap_or(0) != 100 {
                                    continue;
                                }
                            }
                            if filter_perfect {
                                if !completed_games_cache.get(&game.appid).map(|c| c.perfect).unwrap_or(false) {
                                    continue;
                                }
                            }
                            if filter_has_achievements {
                                if !completed_games_cache.get(&game.appid).map(|c| c.has_achievements).unwrap_or(true) {
                                    continue;
                                }
                            }
                            if filter_search != "" {
                                if !game.name.to_lowercase().contains(&filter_search.trim().to_lowercase()) {
                                    continue;
                                }
                            }

                            ui.add_space(5.0);
                            // Add a clickable game using egui::Label::sense()
                            let progress = completed_games_cache.get(&game.appid).map(|c| c.complete).unwrap_or(0).to_string();
                            if ui
                                .add(egui::Label::new(format!("{name} : [{progress}%]", name = &game.name, progress = progress))
                                .sense(egui::Sense::click()))
                                .clicked() {
                                    if !selected_game_app_id.contains(&game.appid) {
                                        selected_game_app_id.insert(game.appid);
                                    }
                                    else {
                                        selected_game_app_id.remove(&game.appid);
                                    }
                            };
                            ui.add_space(5.0);
                        }
                    });
                });
                
            });   
        });
        if refresh {
            goals = get_goals();
            refresh = false;
        }
    })
}

fn get_goals() -> Vec<achievement_store::Achievement> {
    let mut goals: Vec<achievement_store::Achievement> = achievement_store::get_achievements().expect("Failed to load achievements");
    goals.sort_by(|a, b| i32::cmp(&a.app_id,&b.app_id));
    return goals;
}