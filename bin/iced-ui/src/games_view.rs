use super::App;

use crate::Message;

use iced::font;
use iced::widget::{
    center_x, center_y, column, row, table, text, scrollable, button,
};
use iced::{Element, Font};
use db::{
    game_completion_cache::GameCompletion,
};
use api::game_fetch::Game;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub enum GameFilter {
    #[default]
    None,
    InProgress,
    Completed,
    Perfected,
}

pub struct GameDisplay {
    pub game_name: String,
}

impl GameDisplay {
    pub fn list(owned_games: &Vec<Game>, completed_games_cache: &HashMap<i32, GameCompletion>, filter: GameFilter) -> Vec<Self> {
        owned_games
            .iter()
            .filter(|g| {
                match filter {
                    GameFilter::None => true,
                    GameFilter::InProgress => {
                        completed_games_cache.get(&g.appid).map(|c| c.complete).unwrap_or(0) < 100
                    },
                    GameFilter::Completed => {
                        completed_games_cache.get(&g.appid).map(|c| c.complete).unwrap_or(0) == 100
                    },
                    GameFilter::Perfected => {
                        completed_games_cache.get(&g.appid).map(|c| c.perfect).unwrap_or(false)
                    }
                }
            })
            .map(|g| GameDisplay{game_name: g.name.clone()})
            .collect()
    }
}

impl App {
    pub fn game_view(&self) -> Element<'_, Message> {
        let filter_games = {
            row![
                button("In progress").on_press(Message::GamesInProgress).padding(5),
                button("Completed").on_press(Message::GamesCompleted),
                button("Perfected").on_press(Message::GamesPerfected),
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
                table::column(bold("Game Name"), |game: &GameDisplay| text(&game.game_name)),
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