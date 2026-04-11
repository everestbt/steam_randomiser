mod games_view;
mod goals_view;

use iced::widget::{
    center_x, column, row, button,
};
use iced::{Element, Theme};
use games_view::{GameDisplay, GameFilter};
use goals_view::Goal;
use api::game_fetch::{self, Game};
use std::env;
use std::collections::HashMap;
use db::{
    steam_id_store,
    game_completion_cache::{self, GameCompletion},
};
use goals_lib::goals;

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
    games: Vec<GameDisplay>,
    games_have_achievements_filter: bool,
    goals: Vec<Goal>,
    // DATA
    owned_games: Vec<Game>,
    completed_games_cache: HashMap<i32, GameCompletion>,
}

#[derive(Debug, Clone)]
enum Message {
    GamesView,
    GoalsView,
    GamesInProgress,
    GamesCompleted,
    GamesPerfected,
    AchievementCheckboxToggled(bool),
}

#[derive(Debug, Clone, Default)]
enum View {
    Goals,
    #[default]
    Games,
}

impl App {
    fn new() -> Self {
        let data = load_data();
        Self {
            view: View::default(),
            games: GameDisplay::list(&data.owned_games, &data.completed_games_cache, true, GameFilter::default()),
            games_have_achievements_filter: true,
            goals: Goal::list(),
            owned_games: data.owned_games,
            completed_games_cache: data.completed_games_cache,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::GamesView => self.view = View::Games,
            Message::GoalsView => self.view = View::Goals,
            Message::GamesInProgress => self.games = GameDisplay::list(&self.owned_games, &self.completed_games_cache, self.games_have_achievements_filter, GameFilter::InProgress),
            Message::GamesCompleted => self.games = GameDisplay::list(&self.owned_games, &self.completed_games_cache, self.games_have_achievements_filter, GameFilter::Completed),
            Message::GamesPerfected => self.games = GameDisplay::list(&self.owned_games, &self.completed_games_cache, self.games_have_achievements_filter, GameFilter::Perfected),
            Message::AchievementCheckboxToggled(is_checked) => self.games_have_achievements_filter = is_checked,
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

struct DataLoad {
    owned_games: Vec<Game>,
    completed_games_cache: HashMap<i32, GameCompletion>,
}

fn load_data() -> DataLoad {
    let key = env::var("STEAM_API_KEY").expect("You need to set the environment variable STEAM_API_KEY with your API key");
    let steam_id = steam_id_store::get_id().expect("Failed to load steam-id, use the cli and supply a --id first");
    let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    // Sync and update all data
    runtime.block_on(goals::get_and_sync_completed_achievements(&key, &steam_id));
    let owned_games = runtime.block_on(game_fetch::get_owned_games(&key, &steam_id));
    runtime.block_on(goals::refresh_game_completion_cache(&key, &steam_id, &owned_games));
    let completed_games_cache: HashMap<i32, GameCompletion> = game_completion_cache::get_game_completion()
        .expect("Failed to load completed games")
        .iter()
        .map(|n| (n.app_id, n.clone()))
        .collect();
    DataLoad { 
        owned_games,
        completed_games_cache,
    }
}
