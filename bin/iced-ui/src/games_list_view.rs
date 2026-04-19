use super::App;

use crate::{Credentials, Message};

use bytes::Bytes;
use iced::font;
use iced::widget::{
    center_x, center_y, column, row, table, text, scrollable, button, checkbox, image, image::Handle, grid,
};
use iced::{Element, Font};
use db::{
    game_completion_cache,
    game_completion_cache::GameCompletion,
    game_target_store,
};
use api::{
    game_fetch,
    game_fetch::Game,
    game_cover_fetch,
};
use std::collections::{HashMap, HashSet};
use std::cmp::Reverse;
use rayon::prelude::*;

#[derive(Debug, Clone, Default)]
pub enum GameListFilter {
    Targets,
    #[default]
    InProgress,
    Completed,
    Perfected,
}

#[derive(Debug, Clone, Default)]
pub enum GameListView {
    #[default]
    List,
    Grid,
}

#[derive(Debug, Clone)]
pub struct GameListDisplay {
    //DISPLAY
    pub game_name: String,
    pub progress_display: String,
    pub image: Option<Bytes>,
    //DATA
    pub id: i32,
}

impl GameListDisplay {
    pub async fn list(credentials: Credentials, has_achievements: bool, filter: GameListFilter, view_mode: GameListView) -> Vec<Self> {
        let owned_games = game_fetch::get_owned_games(&credentials.key, &credentials.steam_id).await;
        let completed_games_cache: HashMap<i32, GameCompletion> = game_completion_cache::get_game_completion()
        .expect("Failed to load completed games")
        .iter()
        .map(|n| (n.app_id, n.clone()))
        .collect();
        let target_set: HashSet<i32> = game_target_store::get_game_targets().expect("Failed to load targets")
            .iter()
            .filter(|t| !t.complete)
            .map(|t| t.app_id)
            .collect();

        let mut list: Vec<(Game, i8)> = owned_games
            .par_iter()
            .filter(|g| {
                match filter {
                    GameListFilter::Targets => {
                        target_set.contains(&g.appid)
                    }
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
            .map(|g| (g.clone(), completed_games_cache.get(&g.appid).map(|c| c.complete).unwrap_or(0))) // Game, Progress
            .collect();
        list.sort_by_key(|a| Reverse(a.1));

        // If grid only take the first 100 games
        let games = match view_mode {
            GameListView::List => list,
            GameListView::Grid => {
                if list.len() <= 100 {
                    list
                }
                else {
                    list[..99].to_vec()
                }
            },
        };
        games
            .par_iter()
            .map(|g| {
                let image = match view_mode {
                    GameListView::Grid => game_cover_fetch::get_game_cover_blocking(&g.0.appid),
                    GameListView::List => None,
                };
                GameListDisplay{
                    game_name: g.0.name.clone(),
                    progress_display: g.1.to_string(),
                    image,
                    id: g.0.appid,
                }
            })
            .collect()
    }
}

impl App {
    pub fn game_list_view(&self, view_mode: &GameListView) -> Element<'_, Message> {
        let filter_games = {
            row![
                button("Targets").on_press(Message::GamesView(view_mode.clone(), GameListFilter::Targets)),
                button("In progress").on_press(Message::GamesView(view_mode.clone(), GameListFilter::InProgress)),
                button("Completed").on_press(Message::GamesView(view_mode.clone(), GameListFilter::Completed)),
                button("Perfected").on_press(Message::GamesView(view_mode.clone(), GameListFilter::Perfected)),
                button("Grid").on_press(Message::GamesView(GameListView::Grid, GameListFilter::Targets)),
            ]
        };

        let random_game = button("Random Game").on_press(Message::RandomGame);

        let achievement_filter = checkbox(self.games_have_achievements_filter)
            .label("Has Achievements")
            .on_toggle(Message::AchievementCheckboxToggled);
        let game_count = if let Some(games) = &self.games {
            text("Number of games:".to_owned() + games.len().to_string().as_str())
        }
        else {
            text("Loading...")
        };
        let main_view = {
            if let Some (games) = &self.games {
                match view_mode {
                    GameListView::List => {
                        let bold = |header| {
                            text(header).font(Font {
                                weight: font::Weight::Bold,
                                ..Font::DEFAULT
                            })
                        };
                        let columns = [
                            table::column(bold("Game Name"), |game: &GameListDisplay| button(game.game_name.as_str()).on_press(Message::GameView(game.id))),
                            table::column(bold("Progress"), |game: &GameListDisplay| text(game.progress_display.as_str())),
                        ];

                        column![table(columns, games)
                            .padding_x(10)
                            .padding_y(5)
                            .separator_x(1)
                            .separator_y(1)]
                    },
                    GameListView::Grid => {
                        let panes = games.iter().map(|g| {
                            if let Some(i) = &g.image {
                                image(Handle::from_bytes(i.clone())).into()
                            }
                            else {
                                text(g.game_name.clone()).into()
                            }
                        });
                        column![grid(panes)
                            .spacing(10)]
                    },
                }
            }
            else {
                column![text("Loading game list...")]
            }
        };
        column![
            center_x(filter_games).padding(5),
            center_x(achievement_filter).padding(5),
            center_x(random_game).padding(5),
            center_x(game_count).padding(5),
            center_y(scrollable(center_x(main_view)).spacing(10)).padding(10),
        ].into()
    }
}