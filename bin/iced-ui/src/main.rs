mod games_view;
mod goals_view;

use iced::widget::{
    center_x, column, container, row, slider, text, tooltip, button,
};
use iced::{Center, Element, Font, Theme};
use games_view::{Game, GameFilter};
use goals_view::Goal;

pub fn main() -> iced::Result {
    color_eyre::install().expect("Failed to install color eyre");
    iced::application(App::new, App::update, App::view)
        .theme(Theme::CatppuccinMocha)
        .run()
}

struct App {
    view: View,
    games: Vec<Game>,
    goals: Vec<Goal>,
    padding: (f32, f32),
    separator: (f32, f32),
}

#[derive(Debug, Clone)]
enum Message {
    PaddingChanged(f32, f32),
    SeparatorChanged(f32, f32),
    GamesView,
    GoalsView,
    GamesInProgress,
    GamesCompleted,
}

#[derive(Debug, Clone, Default)]
enum View {
    Goals,
    #[default]
    Games,
}

impl App {
    fn new() -> Self {
        Self {
            view: View::default(),
            games: Game::list(GameFilter::default()),
            goals: Goal::list(),
            padding: (10.0, 5.0),
            separator: (1.0, 1.0),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::PaddingChanged(x, y) => self.padding = (x, y),
            Message::SeparatorChanged(x, y) => self.separator = (x, y),
            Message::GamesView => self.view = View::Games,
            Message::GoalsView => self.view = View::Goals,
            Message::GamesInProgress => self.games = Game::list(GameFilter::InProgress),
            Message::GamesCompleted => self.games = Game::list(GameFilter::Completed),
        }
    }

    fn view(&self) -> Element<'_, Message> {

        let view_selector = {
            row![
                button("Games").on_press(Message::GamesView),
                button("Goals").on_press(Message::GoalsView),
            ]
        };

        let main_view = match self.view {
            View::Goals => self.goal_view(),
            View::Games => self.game_view(),
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
            center_x(view_selector).padding(10),
            main_view,
            center_x(controls).padding(10).style(container::dark)
        ]
        .into()
    }
}
