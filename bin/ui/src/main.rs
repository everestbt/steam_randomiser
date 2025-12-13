use api::game_fetch;
use db::achievement_store;

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

    let goals: Vec<achievement_store::Achievement> = achievement_store::get_achievements().expect("Failed to load achievements");
 
    eframe::run_simple_native("Steam randomiser", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to Steam Randomiser");

            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                // Display the first selected game
                let selected = game_list.iter().filter(|a| a.selected);
                ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                    // List out goals for each selected item    
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
                        ui.add_space(5.0);
                    }   
                });      
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                        for game in &mut game_list {
                            ui.add_space(5.0);
                            // Add a clickable game using egui::Label::sense()
                            if ui
                                .add(egui::Label::new(&game.game.name).sense(egui::Sense::click()))
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