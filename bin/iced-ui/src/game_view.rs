use super::App;

use crate::View;
use crate::Message;

use iced::widget::{
    center_x, center_y, column, text, button, table, scrollable
};
use iced::{Center, Left, Element, Font, font};
use api::achievement_fetch;
use std::env;
use std::cmp::Ordering;
use db::{
    steam_id_store,
    game_target_store,
};

#[derive(Debug, Clone)]
pub struct GameDisplay {
    pub game_name: String,
    pub target: bool,
    pub complete: bool,
    goals: Vec<GameGoalDisplay>,
}

#[derive(Debug, Clone)]
struct GameGoalDisplay {
    // DISPLAY
    achievement_name: String,
    description: String,
    // DATA
    complete: bool,
    goal: bool,
}

impl App {
    pub fn game_view(&self) -> Element<'_, Message> {
        match self.view {
            View::Game(app_id) => {
                let game = self.game_views.get(&app_id).expect("Should have been inserted on message processing");
                let game_target_button = {
                    if !game.target {
                        Some(button("Target!").on_press(Message::SetAsGameTarget(app_id)))
                    }
                    else if !game.complete {
                        Some(button("Set as complete!").on_press(Message::SetGameAsComplete(app_id)))
                    }
                    else {
                        None
                    }
                };
                let random_achievement = button("Random achievement!").on_press(Message::GenerateRandomAchievement(app_id));

                let controls = if let Some(target) = game_target_button {
                    column![
                        center_x(target),
                        center_x(random_achievement),
                    ]
                }
                else {
                    column![random_achievement]
                };

                let table = {
                    let bold = |header| {
                        text(header).font(Font {
                            weight: font::Weight::Bold,
                            ..Font::DEFAULT
                        })
                    };
                    let columns = [
                        table::column(bold("Achievements"), |goal: &GameGoalDisplay| text(&goal.achievement_name).style({
                            if goal.complete {
                                text::success
                            }
                            else {
                                if goal.goal {
                                    text::warning
                                }
                                else {
                                    text::default
                                }
                            }
                        }))
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
                    center_x(text(game.game_name.clone())),
                    center_x(controls),
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
            let steam_id = steam_id_store::get_id().expect("Failed to load steam-id, use the cli and supply a --id first");
            let runtime: tokio::runtime::Runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
            let player_achievements = runtime.block_on(achievement_fetch::get_player_achievements(&key, &steam_id, &game.appid));

            let mut goals: Vec<GameGoalDisplay> = runtime.block_on(achievement_fetch::get_game_achievements(&key, &game.appid))
                .iter()
                .map(|a| GameGoalDisplay {
                    achievement_name : a.display_name.clone(),
                    description: a.description.clone().unwrap_or("-".to_string()),
                    complete: player_achievements.as_ref()
                        .map(|p| p.achievements.iter()
                            .find(|pa| pa.apiname == a.name)
                            .map(|pa| pa.achieved == 1))
                            .unwrap_or(None)
                            .unwrap_or(false),
                    goal: self.goals.iter().find(|g| g.game_name == game.name && g.achievement_name == a.display_name).is_some()
                })
                .collect();
            goals.sort_by(|a,b| {
                if a.complete && b.complete {
                    Ordering::Equal
                }
                else if a.complete {
                    Ordering::Greater
                }
                else if b.complete {
                    Ordering::Less
                }
                else {
                    if a.goal && b.goal {
                        Ordering::Equal
                    }
                    else if a.goal {
                        Ordering::Less
                    }
                    else if b.goal {
                        Ordering::Greater
                    }
                    else {
                        Ordering::Equal
                    }
                }
            });
            let target = game_target_store::get_game_target(id).expect("Failed to load target");
            let display = GameDisplay { 
                game_name: game.name.clone(),
                goals,
                target: target.is_some(),
                complete: target.map(|t| t.complete).unwrap_or(false),
            };
            self.game_views.insert(id.clone(), display);
        }
    }
}