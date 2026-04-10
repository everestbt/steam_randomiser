use super::App;

use crate::Message;

use iced::font;
use iced::widget::{
    center_x, center_y, column, row, table, text, scrollable, button,
};
use iced::{Element, Font};
use db::{
    steam_id_store
};
use api::game_fetch;
use std::env;

#[derive(Debug, Clone, Default)]
pub enum GameFilter {
    #[default]
    None,
    InProgress,
    Completed,
}

pub struct Game {
    pub game_name: String,
}

impl Game {
    pub fn list(filter: GameFilter) -> Vec<Self> {
        let key = env::var("STEAM_API_KEY").expect("You need to set the environment variable STEAM_API_KEY with your API key");
        let steam_id = steam_id_store::get_id().expect("Failed to load steam-id, use the cli and supply a --id first");

        let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        runtime.block_on(game_fetch::get_owned_games(&key, &steam_id))
            .into_iter()
            .filter(|_| {
                match filter {
                    GameFilter::None => true,
                    GameFilter::InProgress => true,
                    GameFilter::Completed => true,
                }
            })
            .map(|g| Game{game_name: g.name.clone()})
            .collect()
    }
}

impl App {
    pub fn game_view(&self) -> Element<'_, Message> {
        let filter_games = {
            row![
                button("In progress").on_press(Message::GamesInProgress).padding(5),
                button("Completed").on_press(Message::GamesCompleted),
            ]
        };

        let table = {
            let bold = |header| {
                text(header).font(Font {
                    weight: font::Weight::Bold,
                    ..Font::DEFAULT
                })
            };
            let columns = [
                table::column(bold("Game Name"), |game: &Game| text(&game.game_name)),
            ];

            table(columns, &self.games)
                .padding_x(10)
                .padding_y(5)
                .separator_x(1)
                .separator_y(1)
        };
        column![
            center_x(filter_games).padding(10),
            center_y(scrollable(center_x(table)).spacing(10)).padding(10),
        ].into()
    }
}