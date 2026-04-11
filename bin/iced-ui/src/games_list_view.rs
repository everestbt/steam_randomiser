use super::App;

use crate::Message;

use iced::font;
use iced::widget::{
    center_x, center_y, column, row, table, text, scrollable, button, checkbox
};
use iced::{Element, Font};
use db::{
    game_completion_cache::GameCompletion,
};
use api::game_fetch::Game;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub enum GameListFilter {
    #[default]
    None,
    InProgress,
    Completed,
    Perfected,
}

pub struct GameListDisplay {
    //DISPLAY
    pub game_name: String,
    //DATA
    id: i32,
}

impl GameListDisplay {
    pub fn list(owned_games: &Vec<Game>, completed_games_cache: &HashMap<i32, GameCompletion>, has_achievements: bool, filter: GameListFilter) -> Vec<Self> {
        owned_games
            .iter()
            .filter(|g| {
                match filter {
                    GameListFilter::None => true,
                    GameListFilter::InProgress => {
                        completed_games_cache.get(&g.appid).map(|c| c.complete).unwrap_or(0) < 100
                    },
                    GameListFilter::Completed => {
                        completed_games_cache.get(&g.appid).map(|c| c.complete).unwrap_or(0) == 100
                    },
                    GameListFilter::Perfected => {
                        completed_games_cache.get(&g.appid).map(|c| c.perfect).unwrap_or(false)
                    }
                }
            })
            .filter(|g| {
                if has_achievements {
                    completed_games_cache.get(&g.appid).map(|c| c.has_achievements).unwrap_or(false)
                }
                else {
                    true
                }
            })
            .map(|g| GameListDisplay{
                game_name: g.name.clone(),
                id: g.appid.clone(),
            })
            .collect()
    }
}

impl App {
    pub fn game_list_view(&self) -> Element<'_, Message> {
        let filter_games = {
            row![
                button("In progress").on_press(Message::GamesInProgress).padding(5),
                button("Completed").on_press(Message::GamesCompleted),
                button("Perfected").on_press(Message::GamesPerfected),
            ]
        };

        let achievement_filter = checkbox(self.games_have_achievements_filter)
            .label("Has Achievements")
            .on_toggle(Message::AchievementCheckboxToggled);

        let table = {
            let bold = |header| {
                text(header).font(Font {
                    weight: font::Weight::Bold,
                    ..Font::DEFAULT
                })
            };
            let columns = [
                table::column(bold("Game Name"), |game: &GameListDisplay| button(game.game_name.as_str()).on_press(Message::GameView(game.id).into())),
            ];

            table(columns, &self.games)
                .padding_x(10)
                .padding_y(5)
                .separator_x(1)
                .separator_y(1)
        };
        column![
            center_x(filter_games).padding(10),
            center_x(achievement_filter).padding(10),
            center_y(scrollable(center_x(table)).spacing(10)).padding(10),
        ].into()
    }
}