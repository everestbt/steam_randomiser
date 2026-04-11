use super::App;

use crate::View;
use crate::Message;

use iced::widget::{
    center_x, column, text,
};
use iced::{Element};

#[derive(Debug, Clone)]
pub struct GameDisplay {
    pub game_name: String,
}

impl App {
    pub fn game_view(&self) -> Element<'_, Message> {
        match self.view {
            View::Game(app_id) => {
                let game = self.game_views.get(&app_id).expect("Should have been inserted on message processing");
                column![
                    center_x(text(game.game_name.clone())),
                ].into()
            },
            _ => unreachable!("Only called when a game view")
        }
    }

    pub fn load_game_display(&mut self, id: &i32) {
        if !self.game_views.contains_key(id) {
            let game = self.owned_games.iter().find(|o| &o.appid == id).expect("Selected a game that does not exist");
            let display = GameDisplay { game_name: game.name.clone() };
            self.game_views.insert(id.clone(), display);
        }
    }
}