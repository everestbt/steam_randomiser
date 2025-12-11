use api::game_fetch;

use eframe::egui;
use std::env;

fn main() -> eframe::Result {
    let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");

    let args: Vec<String> = env::args().collect();
    let key: String = args[1].clone();
    let steam_id: String = args[2].clone();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 1200.0]),
        ..Default::default()
    };

    let mut owned_games: Vec<game_fetch::Game> = runtime.block_on(game_fetch::get_owned_games(&key, &steam_id));
    owned_games.sort_by(|a,b| a.name.cmp(&b.name));

    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to Steam Randomiser");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for game in &owned_games {
                    // Add some spacing to let it breathe
                    ui.add_space(5.0);

                    // Add a clickable label using egui::Label::sense()
                    if ui
                        .add(egui::Label::new(&game.name).sense(egui::Sense::click()))
                        .clicked() {
                    };

                    // Add some spacing to let it breathe
                    ui.add_space(5.0);
                }
            });
        });
    })
}