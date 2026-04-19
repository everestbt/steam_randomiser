mod games_list_view;
mod goals_view;
mod game_view;

use iced::widget::{
    center_x, column, row, button,
};
use iced::{Element, Theme, Task};
use games_list_view::{GameListDisplay, GameListFilter, GameListView};
use goals_view::Goal;
use api::game_fetch::{self, Game};
use simple_error::SimpleError;
use std::env;
use std::collections::HashMap;
use db::{
    steam_id_store,
    game_target_store,
    excluded_achievement_store,
};
use goals_lib::goals;
use game_view::GameDisplay;
use api::achievement_fetch::GameAchievement;

pub fn main() -> iced::Result {
    color_eyre::install().expect("Failed to install color eyre");
    iced::application(App::new, App::update, App::view)
        .theme(Theme::CatppuccinMocha)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    GamesView(GameListView, GameListFilter),
    GameView(i32), //app_id
    GoalsView,
    AchievementCheckboxToggled(bool),
    GamesLoaded(Vec<GameListDisplay>),
    GenerateRandomAchievement(i32), // app_id
    RandomAchievementGenerated(Result<(Game, Option<GameAchievement>), SimpleError>), 
    SetAsGameTarget(i32), // app_id
    SetGameAsComplete(i32), // app_id
    RandomGame,
    ExcludeAchievement(i32, String) // app_id, achievement_name
}

#[derive(Debug, Clone)]
enum View {
    Goals,
    Games(GameListView, GameListFilter),
    Game(i32), // app_id
}

#[derive(Debug, Clone)]
struct Credentials {
    key: String,
    steam_id: String,
}

struct App {
    // SETTINGS
    view: View,
    // DISPLAY
    games: Option<Vec<GameListDisplay>>,
    games_have_achievements_filter: bool,
    goals: Vec<Goal>,
    game_views : HashMap<i32, GameDisplay>,
    // DATA
    credentials: Credentials,
    owned_games: Vec<Game>,
}

impl App {
    fn new() -> Self {
        let data = load_data();
        Self {
            view: View::Goals,
            games: None,
            games_have_achievements_filter: true,
            goals: Goal::list(),
            game_views: HashMap::new(),
            credentials: data.credentials,
            owned_games: data.owned_games,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GamesView(list_view, filter) => {
                self.view = View::Games(list_view.clone(), filter.clone());
                self.games = None;
                Task::perform(GameListDisplay::list(self.credentials.clone(), self.games_have_achievements_filter, filter.clone(), list_view.clone()), Message::GamesLoaded)
            },
            Message::GameView(id) => {
                self.load_game_display(&id);
                self.view = View::Game(id);
                Task::none()
            },
            Message::GoalsView => {
                self.view = View::Goals;
                Task::none()
            },
            Message::AchievementCheckboxToggled(is_checked) => {
                self.games_have_achievements_filter = is_checked;
                match &self.view {
                    View::Games(list_view, filter) => {
                        self.games = None;
                        Task::perform(GameListDisplay::list(self.credentials.clone(), self.games_have_achievements_filter, filter.clone(), list_view.clone()), Message::GamesLoaded)
                    },
                    _ => Task::none()
                }
                
            },
            Message::GamesLoaded(loaded) => {
                self.games = Some(loaded);
                Task::none()
            },
            Message::GenerateRandomAchievement(ref app_id) => Task::perform(game_view::generate_random_achievement(self.credentials.clone(), app_id.clone()), Message::RandomAchievementGenerated),
            Message::RandomAchievementGenerated(random_achievement) => {
                if let Ok(r) = random_achievement {
                    self.handle_generated_random_achievement(r.0, r.1);
                }
                else {
                    panic!("{}", random_achievement.unwrap_err().as_str())
                }
                Task::none()
            },
            Message::SetAsGameTarget(app_id) => {
                game_target_store::save_game_target(&app_id, &false).expect("Failed to save target");
                if let Some(view) = self.game_views.get_mut(&app_id) {
                    view.target = true;
                }
                Task::none()
            },
            Message::SetGameAsComplete(app_id) => {
                game_target_store::save_game_target(&app_id, &true).expect("Failed to save target");
                if let Some(view) = self.game_views.get_mut(&app_id) {
                    view.complete = true;
                }
                Task::none()
            },
            Message::RandomGame => {
                let random_game_id = self.owned_games.get(rand::random_range(..self.owned_games.len())).unwrap().appid;
                self.load_game_display(&random_game_id);
                self.view = View::Game(random_game_id);
                Task::none()
            },
            Message::ExcludeAchievement(app_id, achievement_name) => {
                excluded_achievement_store::save_excluded_achievement(&achievement_name, &app_id).expect("Failed to exclude achievement");
                if let Some(game_view) = self.game_views.get_mut(&app_id) {
                    if let Some(achievement) = game_view.goals.iter_mut().find(|a| a.achievement_name == achievement_name) {
                        achievement.goal_state = game_view::GoalState::Excluded;
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let view_selector = {
            row![
                button("Games").on_press(Message::GamesView(GameListView::default(), GameListFilter::default())),
                button("Goals").on_press(Message::GoalsView),
            ]
        };

        let main_view = match &self.view {
            View::Goals => self.goal_view(),
            View::Games(view, _) => self.game_list_view(view),
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
    credentials: Credentials,
    owned_games: Vec<Game>,
}

fn load_data() -> DataLoad {
    let key = env::var("STEAM_API_KEY").expect("You need to set the environment variable STEAM_API_KEY with your API key");
    let steam_id = steam_id_store::get_id().expect("Failed to load steam-id, use the cli and supply a --id first");
    let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    // Sync and update all data
    runtime.block_on(goals::get_and_sync_completed_achievements(&key, &steam_id));
    let owned_games = runtime.block_on(game_fetch::get_owned_games(&key, &steam_id));
    runtime.block_on(goals::refresh_game_completion_cache(&key, &steam_id, &owned_games));
    DataLoad { 
        credentials: Credentials { key, steam_id },
        owned_games,
    }
}
