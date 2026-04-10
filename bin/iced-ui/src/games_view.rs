use super::App;

use crate::Message;

use iced::font;
use iced::widget::{
    table, text,
};
use iced::{Font};
use db::{
    steam_id_store
};
use api::game_fetch;
use std::env;

pub struct Game {
    pub game_name: String,
}

impl Game {
    pub fn list() -> Vec<Self> {
        let key = env::var("STEAM_API_KEY").expect("You need to set the environment variable STEAM_API_KEY with your API key");
        let steam_id = steam_id_store::get_id().expect("Failed to load steam-id, use the cli and supply a --id first");

        let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        runtime.block_on(game_fetch::get_owned_games(&key, &steam_id))
            .into_iter()
            .map(|g| Game{game_name: g.name.clone()})
            .collect()
    }
}

impl App {
    pub fn game_table(&self) -> table::Table<'_, Message> {
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
            .padding_x(self.padding.0)
            .padding_y(self.padding.1)
            .separator_x(self.separator.0)
            .separator_y(self.separator.1)
    }
}