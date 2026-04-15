mod games_list_view;
mod goals_view;
mod game_view;

use iced::widget::{
    center_x, column, row, button,
};
use iced::{Element, Theme};
use games_list_view::{GameListDisplay, GameListFilter};
use goals_view::Goal;
use api::game_fetch::{self, Game};
use std::env;
use std::collections::HashMap;
use db::{
    steam_id_store,
    game_completion_cache::{self, GameCompletion},
    achievement_store,
    game_target_store,
    excluded_achievement_store,
};
use goals_lib::goals;
use game_view::GameDisplay;

pub fn main() -> iced::Result {
    color_eyre::install().expect("Failed to install color eyre");
    iced::application(App::new, App::update, App::view)
        .theme(Theme::CatppuccinMocha)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    GamesView,
    GameView(i32), //app_id
    GoalsView,
    GamesTargets,
    GamesInProgress,
    GamesCompleted,
    GamesPerfected,
    AchievementCheckboxToggled(bool),
    GenerateRandomAchievement(i32), // app_id
    SetAsGameTarget(i32), // app_id
    SetGameAsComplete(i32), // app_id
    RandomGame,
    ExcludeAchievement(i32, String) // app_id, achievement_name
}

#[derive(Debug, Clone, Default)]
enum View {
    Goals,
    #[default]
    Games,
    Game(i32), // app_id
}

struct App {
    // SETTINGS
    view: View,
    // DISPLAY
    games: Vec<GameListDisplay>,
    games_have_achievements_filter: bool,
    goals: Vec<Goal>,
    game_views : HashMap<i32, GameDisplay>,
    // DATA
    owned_games: Vec<Game>,
    completed_games_cache: HashMap<i32, GameCompletion>,
}

impl App {
    fn new() -> Self {
        let data = load_data();
        Self {
            view: View::default(),
            games: GameListDisplay::list(&data.owned_games, &data.completed_games_cache, true, GameListFilter::default()),
            games_have_achievements_filter: true,
            goals: Goal::list(),
            game_views: HashMap::new(),
            owned_games: data.owned_games,
            completed_games_cache: data.completed_games_cache,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::GamesView => self.view = View::Games,
            Message::GameView(id) => {
                self.load_game_display(&id);
                self.view = View::Game(id);
            },
            Message::GoalsView => self.view = View::Goals,
            Message::GamesTargets => self.games = GameListDisplay::list(&self.owned_games, &self.completed_games_cache, self.games_have_achievements_filter, GameListFilter::Targets),
            Message::GamesInProgress => self.games = GameListDisplay::list(&self.owned_games, &self.completed_games_cache, self.games_have_achievements_filter, GameListFilter::InProgress),
            Message::GamesCompleted => self.games = GameListDisplay::list(&self.owned_games, &self.completed_games_cache, self.games_have_achievements_filter, GameListFilter::Completed),
            Message::GamesPerfected => self.games = GameListDisplay::list(&self.owned_games, &self.completed_games_cache, self.games_have_achievements_filter, GameListFilter::Perfected),
            Message::AchievementCheckboxToggled(is_checked) => {
                self.games_have_achievements_filter = is_checked;
            },
            Message::GenerateRandomAchievement(app_id) => {
                let game = self.owned_games.iter().find(|g| g.appid == app_id).expect("Selected for a game that does not exist");
                generate_random_achievement(game);
                self.goals = Goal::list();
            },
            Message::SetAsGameTarget(app_id) => {
                game_target_store::save_game_target(&app_id, &false).expect("Failed to save target");
                if let Some(view) = self.game_views.get_mut(&app_id) {
                    view.target = true;
                }
            },
            Message::SetGameAsComplete(app_id) => {
                game_target_store::save_game_target(&app_id, &true).expect("Failed to save target");
                if let Some(view) = self.game_views.get_mut(&app_id) {
                    view.complete = true;
                }
            },
            Message::RandomGame => {
                let random_game_id = self.games.get(rand::random_range(..self.games.len())).unwrap().id.clone();
                self.load_game_display(&random_game_id);
                self.view = View::Game(random_game_id);
            },
            Message::ExcludeAchievement(app_id, achievement_name) => {
                excluded_achievement_store::save_excluded_achievement(&achievement_name, &app_id).expect("Failed to exclude achievement");
                if let Some(game_view) = self.game_views.get_mut(&app_id) {
                    if let Some(achievement) = game_view.goals.iter_mut().find(|a| a.achievement_name == achievement_name) {
                        achievement.goal_state = game_view::GoalState::Excluded;
                    }
                }
            }
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
            View::Games => self.game_list_view(),
            View::Game(_) => self.game_view(),
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

fn generate_random_achievement(game: &Game) {
    let key = env::var("STEAM_API_KEY").expect("You need to set the environment variable STEAM_API_KEY with your API key");
    let steam_id = steam_id_store::get_id().expect("Failed to load steam-id, use the cli and supply a --id first");
    let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let random_achievement = runtime.block_on(goals::get_random_achievement_for_game(&key, &steam_id, game));
    if let Some(a) = random_achievement {
        achievement_store::save_achievement(&a.name, &a.display_name, &a.description, &game.appid, &game.last_played).expect("Failed to save achievement");
    }
}
