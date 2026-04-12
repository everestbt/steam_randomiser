use super::App;

use crate::View;
use crate::Message;

use iced::widget::{
    center_x, center_y, column, text, button, table, scrollable
};
use iced::{Center, Left, Element, Font, font};
use api::achievement_fetch;
use std::env;

#[derive(Debug, Clone)]
pub struct GameDisplay {
    pub game_name: String,
    goals: Vec<GameGoalDisplay>,
}

#[derive(Debug, Clone)]
struct GameGoalDisplay {
    achievement_name: String,
    description: String,
}

impl App {
    pub fn game_view(&self) -> Element<'_, Message> {
        match self.view {
            View::Game(app_id) => {
                let game = self.game_views.get(&app_id).expect("Should have been inserted on message processing");
                let random_achievement = button("Random achievement!").on_press(Message::GenerateRandomAchievement(app_id));

                let table = {
                    let bold = |header| {
                        text(header).font(Font {
                            weight: font::Weight::Bold,
                            ..Font::DEFAULT
                        })
                    };
                    let columns = [
                        table::column(bold("Achievements"), |goal: &GameGoalDisplay| text(&goal.achievement_name))
                            .align_x(Left)
                            .align_y(Center),
                        table::column(bold("Description"), |goal: &GameGoalDisplay| text(&goal.description))
                            .align_x(Left)
                            .align_y(Center),
                    ];

                    table(columns, &game.goals)
                        .padding_x(10)
                        .padding_y(5)
                        .separator_x(1)
                        .separator_y(1)
                };

                column![
                    center_x(random_achievement),
                    center_x(text(game.game_name.clone())),
                    center_y(scrollable(center_x(table)).spacing(10)).padding(10),
                ].into()
            },
            _ => unreachable!("Only called when a game view")
        }
    }

    pub fn load_game_display(&mut self, id: &i32) {
        if !self.game_views.contains_key(id) {
            let game = self.owned_games.iter().find(|o| &o.appid == id).expect("Selected a game that does not exist");
            let key = env::var("STEAM_API_KEY").expect("You need to set the environment variable STEAM_API_KEY with your API key");
            let runtime: tokio::runtime::Runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
            let goals = runtime.block_on(achievement_fetch::get_game_achievements(&key, &game.appid))
                .iter()
                .map(|a| GameGoalDisplay {
                    achievement_name : a.display_name.clone(),
                    description: a.description.clone().unwrap_or("-".to_string()),
                })
                .collect();
            let display = GameDisplay { 
                game_name: game.name.clone(),
                goals
            };
            self.game_views.insert(id.clone(), display);
        }
    }
}