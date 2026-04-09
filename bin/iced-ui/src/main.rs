use iced::font;
use iced::widget::{
    center_x, center_y, column, container, row, scrollable, slider, table, text, tooltip,
};
use iced::{Center, Left, Element, Font, Theme};
use db::{
    achievement_store, 
    steam_id_store
};
use api::game_fetch;
use std::collections::HashMap;
use std::env;

pub fn main() -> iced::Result {
    color_eyre::install().expect("Failed to install color eyre");
    iced::application(Table::new, Table::update, Table::view)
        .theme(Theme::CatppuccinMocha)
        .run()
}

struct Table {
    events: Vec<Goal>,
    padding: (f32, f32),
    separator: (f32, f32),
}

#[derive(Debug, Clone)]
enum Message {
    PaddingChanged(f32, f32),
    SeparatorChanged(f32, f32),
}

impl Table {
    fn new() -> Self {
        Self {
            events: Goal::list(),
            padding: (10.0, 5.0),
            separator: (1.0, 1.0),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::PaddingChanged(x, y) => self.padding = (x, y),
            Message::SeparatorChanged(x, y) => self.separator = (x, y),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let table = {
            let bold = |header| {
                text(header).font(Font {
                    weight: font::Weight::Bold,
                    ..Font::DEFAULT
                })
            };

            let columns = [
                table::column(bold("Game Name"), |event: &Goal| text(&event.game_name)),
                table::column(bold("Achievement Name"), |event: &Goal| text(&event.achievement_name))
                    .align_x(Left)
                    .align_y(Center),
                table::column(bold("Description"), |event: &Goal| text(&event.description))
                    .align_x(Left)
                    .align_y(Center),
            ];

            table(columns, &self.events)
                .padding_x(self.padding.0)
                .padding_y(self.padding.1)
                .separator_x(self.separator.0)
                .separator_y(self.separator.1)
        };

        let controls = {
            let labeled_slider =
                |label,
                 range: std::ops::RangeInclusive<f32>,
                 (x, y),
                 on_change: fn(f32, f32) -> Message| {
                    row![
                        text(label).font(Font::MONOSPACE).size(14).width(100),
                        tooltip(
                            slider(range.clone(), x, move |x| on_change(x, y)),
                            text!("{x:.0}px").font(Font::MONOSPACE).size(10),
                            tooltip::Position::Left
                        ),
                        tooltip(
                            slider(range, y, move |y| on_change(x, y)),
                            text!("{y:.0}px").font(Font::MONOSPACE).size(10),
                            tooltip::Position::Right
                        ),
                    ]
                    .spacing(10)
                    .align_y(Center)
                };

            column![
                labeled_slider("Padding", 0.0..=30.0, self.padding, Message::PaddingChanged),
                labeled_slider(
                    "Separator",
                    0.0..=5.0,
                    self.separator,
                    Message::SeparatorChanged
                )
            ]
            .spacing(10)
            .width(400)
        };

        column![
            center_y(scrollable(center_x(table)).spacing(10)).padding(10),
            center_x(controls).padding(10).style(container::dark)
        ]
        .into()
    }
}

struct Goal {
    game_name: String,
    achievement_name: String,
    description: String,
}

impl Goal {
    fn list() -> Vec<Self> {
        let key = env::var("STEAM_API_KEY").expect("You need to set the environment variable STEAM_API_KEY with your API key");
        let steam_id = steam_id_store::get_id().expect("Failed to load steam-id, use the cli and supply a --id first");

        let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        let game_map = runtime.block_on(game_fetch::get_owned_games(&key, &steam_id))
            .into_iter()
            .map(|g| (g.appid.clone(), g))
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