use super::App;

use crate::View;
use crate::Message;
use crate::goals_view::Goal;

use db::excluded_achievement_store;
use iced::widget::{
    center_x, center_y, column, text, button, table, scrollable, image, image::Handle
};
use iced::{Center, Left, Element, Font, font};
use api::achievement_fetch;
use std::env;
use std::collections::HashSet;
use db::{
    steam_id_store,
    game_target_store,
    achievement_store,
};
use rayon::prelude::*;
use bytes::Bytes;
use goals_lib::goals;

#[derive(Debug, Clone)]
pub struct GameDisplay {
    pub game_name: String,
    pub target: bool,
    pub complete: bool,
    pub goals: Vec<GameGoalDisplay>,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum GoalState {
    Goal,
    Incomplete,
    Complete,
    Excluded,
}

#[derive(Debug, Clone)]
pub struct GameGoalDisplay {
    // DISPLAY
    display_name: String,
    description: String,
    image: Bytes,
    // DATA
    pub goal_state: GoalState,
    pub achievement_name: String,
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
                        table::column(bold("Icon"), |goal: &GameGoalDisplay| image(Handle::from_bytes(goal.image.clone())))
                            .align_x(Left)
                            .align_y(Center),
                        table::column(bold("Achievement"), |goal: &GameGoalDisplay| text(&goal.display_name).style({
                            match goal.goal_state {
                                GoalState::Complete => text::success,
                                GoalState::Incomplete => text::default,
                                GoalState::Goal => text::warning,
                                GoalState::Excluded => text::danger,
                            }
                        }))
                            .align_x(Left)
                            .align_y(Center),
                        table::column(bold("Description"), |goal: &GameGoalDisplay| text(&goal.description))
                            .align_x(Left)
                            .align_y(Center),
                        table::column(bold("Exclude"), |goal: &GameGoalDisplay| button("Exclude").on_press(Message::ExcludeAchievement(app_id, goal.achievement_name.clone())))
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
            let excluded_achievements: HashSet<String> = excluded_achievement_store::get_excluded_achievements_for_app(id).expect("Failed to load excluded achievements")
                .iter()
                .map(|a| a.achievement_name.clone())
                .collect();

            let mut goals: Vec<GameGoalDisplay> = runtime.block_on(achievement_fetch::get_game_achievements(&key, &game.appid))
                .par_iter()
                .map(|a| {
                    let goal_state = {
                        if player_achievements.as_ref()
                            .map(|p| p.achievements.iter()
                                .find(|pa| pa.apiname == a.name)
                                .map(|pa| pa.achieved == 1))
                                .unwrap_or(None)
                                .unwrap_or(false) {
                            GoalState::Complete
                        }
                        else if excluded_achievements.contains(&a.name) {
                            GoalState::Excluded
                        }
                        else if self.goals.iter().find(|g| g.game_name == game.name && g.achievement_name == a.display_name).is_some() {
                            GoalState::Goal
                        }
                        else {
                            GoalState::Incomplete
                        }
                    };

                    let img_bytes = if goal_state == GoalState::Complete {
                        reqwest::blocking::get(a.icon.clone()).expect("Failed to load url").bytes().expect("Failed to read bytes")
                    }
                    else {
                        reqwest::blocking::get(a.icongray.clone()).expect("Failed to load url").bytes().expect("Failed to read bytes")
                    };

                    GameGoalDisplay {
                        display_name : a.display_name.clone(),
                        description: a.description.clone().unwrap_or("-".to_string()),
                        image: img_bytes,
                        goal_state,
                        achievement_name: a.name.clone(),
                    }
                })
                .collect();
            goals.sort_by_key(|g| g.goal_state);
            let target = game_target_store::get_game_target(id).expect("Failed to load target");
            let display = GameDisplay { 
                game_name: game.name.clone(),
                goals,
                target: target.is_some(),
                complete: target.map(|t| t.complete).unwrap_or(false),
            };
            self.game_views.insert(*id, display);
        }
    }

    pub fn generate_random_achievement(&mut self, app_id: &i32) {
        let game = self.owned_games.iter().find(|g| &g.appid == app_id).expect("Selected for a game that does not exist");
        let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        let random_achievement = runtime.block_on(goals::get_random_achievement_for_game(&self.credentials.key, &self.credentials.steam_id, game));
        if let Some(ra) = random_achievement {
            achievement_store::save_achievement(&ra.name, &ra.display_name, &ra.description, &game.appid, &game.last_played).expect("Failed to save achievement");
            self.goals = Goal::list();
            if let Some(game_view) = self.game_views.get_mut(app_id) {
                if let Some(achievement) = game_view.goals.iter_mut().find(|a| a.achievement_name == ra.name) {
                    achievement.goal_state = GoalState::Goal;
                }
            }
        }
    }
}