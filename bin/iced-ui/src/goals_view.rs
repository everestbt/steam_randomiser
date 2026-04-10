use super::App;

use crate::Message;

use iced::font;
use iced::widget::{
    table, text, center_x, center_y, column, scrollable
};
use iced::{Center, Left, Font, Element};
use db::{
    achievement_store, 
    steam_id_store
};
use api::game_fetch;
use std::collections::HashMap;
use std::env;


pub struct Goal {
    pub game_name: String,
    pub achievement_name: String,
    pub description: String,
}

impl Goal {
    pub fn list() -> Vec<Self> {
        let key = env::var("STEAM_API_KEY").expect("You need to set the environment variable STEAM_API_KEY with your API key");
        let steam_id = steam_id_store::get_id().expect("Failed to load steam-id, use the cli and supply a --id first");

        let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        let game_map = runtime.block_on(game_fetch::get_owned_games(&key, &steam_id))
            .into_iter()
            .map(|g| (g.appid, g))
            .collect::<HashMap<_, _>>();

        let mut goals = achievement_store::get_achievements().expect("Failed to load achievements");
        goals.sort_by(|a, b| i32::cmp(&a.app_id,&b.app_id));
        goals.iter().map(|g| Goal {
                game_name: game_map.get(&g.app_id).unwrap().name.clone(),
                achievement_name: g.display_name.clone(),
                description: g.description.clone().unwrap_or("-".to_string()),
            })
            .collect()
    }
}

impl App {
    pub fn goal_view(&self) -> Element<'_, Message> {
        let table = {
            let bold = |header| {
                text(header).font(Font {
                    weight: font::Weight::Bold,
                    ..Font::DEFAULT
                })
            };
            let columns = [
                table::column(bold("Game Name"), |goal: &Goal| text(&goal.game_name)),
                table::column(bold("Achievement Name"), |goal: &Goal| text(&goal.achievement_name))
                    .align_x(Left)
                    .align_y(Center),
                table::column(bold("Description"), |goal: &Goal| text(&goal.description))
                    .align_x(Left)
                    .align_y(Center),
            ];

            table(columns, &self.goals)
                .padding_x(10)
                .padding_y(5)
                .separator_x(1)
                .separator_y(1)
        };

        column![
            center_y(scrollable(center_x(table)).spacing(10)).padding(10),
        ].into()
    }
}
