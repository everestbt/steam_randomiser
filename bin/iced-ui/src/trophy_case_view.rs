use super::App;

use crate::Message;

use api::game_cover_fetch;
use iced::{Element};
use iced::widget::{
    column, text, image, image::Handle, grid, scrollable
};
use db::game_completion_cache;
use std::collections::HashMap;
use rayon::prelude::*;

impl App {
    pub fn trophy_case_view(&self) -> Element<'_, Message> {
        if let Some(trophies) = &self.trophies {
            let panes = trophies.iter().map(|app_id| {
                if let Some(i) =  self.game_covers.get(app_id) {
                    image(i).width(150).height(225).into()
                }
                else if let Some(game) = self.owned_games.get(app_id) {
                    text(game.name.clone()).into()
                }
                else {
                    text("Loading").into()
                }
            });
            column![scrollable(grid(panes)
                .spacing(10))].into()
        }
        else {
            column![text("Loading trophy list...")].into()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum TrophyCaseView {
    #[default]
    Completed,
    Perfected,
}

pub async fn load_trophies(view: TrophyCaseView) -> Vec<i32> {
    game_completion_cache::get_game_completion()
        .expect("Failed to load cache")
        .iter()
        .filter(|c| {
            match view {
                TrophyCaseView::Completed => c.complete == 100,
                TrophyCaseView::Perfected => c.perfect
            }
        }) 
        .map(|c| c.app_id)
        .collect()
}

pub async fn load_game_covers(app_ids: Vec<i32>) -> HashMap<i32, Handle> {
    app_ids.par_iter()
        .map(|g| {
            (g.clone(), game_cover_fetch::get_game_cover_blocking(g).map(|b| Handle::from_bytes(b)))
        })
        .filter(|t| t.1.is_some())
        .map(|t| (t.0, t.1.expect("All none will be filtered out")))
        .collect::<HashMap<_, _>>()
}