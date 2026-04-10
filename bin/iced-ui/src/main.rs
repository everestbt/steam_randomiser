mod games_view;
mod goals_view;

use iced::widget::{
    center_x, column, row, button,
};
use iced::{Element, Theme};
use games_view::{Game, GameFilter};
use goals_view::Goal;

pub fn main() -> iced::Result {
    color_eyre::install().expect("Failed to install color eyre");
    iced::application(App::new, App::update, App::view)
        .theme(Theme::CatppuccinMocha)
        .run()
}

struct App {
    // SETTINGS
    view: View,
    // DISPLAY
    games: Vec<Game>,
    goals: Vec<Goal>,
}

#[derive(Debug, Clone)]
enum Message {
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
        }
    }

    fn update(&mut self, message: Message) {
        match message {
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

        column![
            center_x(view_selector).padding(10),
            main_view,
        ]
        .into()
    }
}
