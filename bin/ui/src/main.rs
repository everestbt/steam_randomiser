use api::game_fetch;

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

    // bool == selected
    let mut game_list: Vec<GameListItem> = runtime.block_on(game_fetch::get_owned_games(&key, &steam_id)).iter().map(|g| GameListItem { game: g.clone(), selected: false }).collect();
    game_list.sort_by(|a,b| a.game.name.cmp(&b.game.name));
 
    eframe::run_simple_native("Steam randomiser", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to Steam Randomiser");
            // Display the first selected game
            let selected = game_list.iter().filter(|a| a.selected);
            for s in selected {
                ui.add(egui::Label::new(s.game.name.clone()));
            }            
            egui::ScrollArea::vertical().show(ui, |ui| {
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
    })
}