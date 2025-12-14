use api::game_fetch;
use db::achievement_store;
use goals_lib::goals;

use eframe::egui;
use std::env;

#[derive(Clone)]
struct GameListItem {
    game: game_fetch::Game,
    selected: bool,
}

impl GameListItem {
    fn select(&mut self) {
        self.selected = !self.selected;
    }
}

fn main() -> eframe::Result {
    let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");

    let args: Vec<String> = env::args().collect();
    let key: String = args[1].clone();
    let steam_id: String = args[2].clone();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 1200.0]),
        ..Default::default()
    };

    let mut game_list: Vec<GameListItem> = runtime.block_on(game_fetch::get_owned_games(&key, &steam_id)).iter().map(|g| GameListItem { game: g.clone(), selected: false }).collect();
    game_list.sort_by(|a,b| a.game.name.cmp(&b.game.name));

    let mut goals: Vec<achievement_store::Achievement> = get_goals();
 
    eframe::run_simple_native("Steam randomiser", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to Steam Randomiser");

            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {               
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    // List out all goals
                    ui.heading("Goals");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            for g in &goals {
                                let result;
                                let game_name = game_list.iter().find(|game| game.game.appid == g.app_id).unwrap().game.name.clone();
                                if g.description.is_some() {
                                    result = format!("{} : {} : {}", game_name, g.display_name.clone(), g.description.clone().unwrap());
                                }
                                else {
                                    result = format!("{} : {}", game_name, g.display_name.clone());
                                }
                                ui.label(result);
                                ui.add_space(5.0);
                            }
                        });
                    });
                });  
                // List out goals for each selected items     
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.heading("Selected");
                    let selected = game_list.iter().filter(|a| a.selected);
                    for s in selected {
                        ui.add(egui::Label::new(s.game.name.clone()));
                        ui.add_space(5.0);
                        // Goals
                        ui.label("Goals:");
                        ui.add_space(5.0);
                        let game_goals = goals.iter().filter(|a| a.app_id == s.game.appid);
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
                        if ui.add(egui::Button::new("Click me")).clicked() {
                            let random_achievement = runtime.block_on(goals::get_random_achievement_for_game(&key, &steam_id, &s.game));
                            if random_achievement.is_some() {
                                let a = random_achievement.unwrap();
                                achievement_store::save_achievement(&a.name, &a.display_name, &a.description, &s.game.appid, &s.game.last_played).expect("Failed to save achievement");
                                goals = get_goals();
                            }
                        }
                        ui.add_space(5.0);
                    }   
                });
                // Display a list of every owned game that can then be selected/deselected
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                        for game in &mut game_list {
                            ui.add_space(5.0);
                            // Add a clickable game using egui::Label::sense()
                            if ui
                                .add(egui::Label::new(&game.game.name)
                                .sense(egui::Sense::click()))
                                .clicked() {
                                    game.select();
                            };
                            ui.add_space(5.0);
                        }
                    });
                });
            });   
        });
    })
}

fn get_goals() -> Vec<achievement_store::Achievement> {
    let mut goals: Vec<achievement_store::Achievement> = achievement_store::get_achievements().expect("Failed to load achievements");
    goals.sort_by(|a, b| i32::cmp(&a.app_id,&b.app_id));
    return goals;
}