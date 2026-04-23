mod games_list_view;
mod goals_view;
mod game_view;

use iced::widget::{
    center_x, column, row, button, image::Handle, text
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
use game_view::{GameDisplay, GameGoalDisplay};
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
    GameLoaded(GameDisplay),
    GoalIconsLoaded(HashMap<(i32, String), Handle>), // app_id, achievement_name -> Image
    GoalsView,
    GoalsLoaded(Vec<Goal>),
    AchievementCheckboxToggled(bool),
    GamesLoaded(Vec<GameListDisplay>),
    GenerateRandomAchievement(i32), // app_id
    RandomAchievementGenerated(Result<(Game, Option<GameAchievement>), SimpleError>), 
    SetAsGameTarget(i32), // app_id
    SetGameAsComplete(i32), // app_id
    RandomGame,
    ExcludeAchievement(i32, String) // app_id, achievement_name
}

#[derive(Debug, Clone, Default)]
enum View {
    #[default]
    None,
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
    goals: Option<Vec<Goal>>,
    game_views: HashMap<i32, GameDisplay>,
    goal_icons: HashMap<(i32, String), Handle>, // app_id, achievement_name -> image
    // DATA
    credentials: Credentials,
    owned_games: HashMap<i32, Game>, //app_id -> Game
}

impl App {
    fn new() -> Self {
        let data = load_data();
        Self {
            view: View::default(),
            games: None,
            games_have_achievements_filter: true,
            goals: None,
            game_views: HashMap::new(),
            goal_icons: HashMap::new(),
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
            Message::GamesLoaded(loaded) => {
                self.games = Some(loaded);
                Task::none()
            },
            Message::GameView(id) => {
                self.view = View::Game(id);
                Task::perform(game_view::load_game_display(self.credentials.clone(), id.clone(), self.owned_games.get(&id).expect("Does not exist").name.clone()), Message::GameLoaded)
            },
            Message::GameLoaded(display) => {
                let filtered_icons: Vec<GameGoalDisplay> = display.goals.iter()
                    .filter(|i| !self.goal_icons.contains_key(&(display.app_id, i.achievement_name.clone())))
                    .map(|d| d.clone())
                    .collect();
                let task = if filtered_icons.is_empty() {
                    Task::none()
                } 
                else {
                    Task::perform(game_view::load_all_goal_icons(display.app_id.clone(), filtered_icons), Message::GoalIconsLoaded)
                };
                self.game_views.insert(display.app_id.clone(), display);
                task
            },
            Message::GoalIconsLoaded(icons) => {
                for icon in icons {
                    self.goal_icons.insert(icon.0, icon.1);
                }
                Task::none()
            },
            Message::GoalsView => {
                self.view = View::Goals;
                if self.goals.is_none() {
                    Task::perform(Goal::list(self.credentials.clone()), Message::GoalsLoaded)
                }
                else {
                    Task::none()
                }
            },
            Message::GoalsLoaded(goals) => {
                let mut tasks: Vec<Task<Message>> = Vec::new();
                for g in &goals {
                    tasks.push(Task::perform(game_view::load_game_display(self.credentials.clone(), g.app_id.clone(), g.game_name.clone()), Message::GameLoaded));
                }
                self.goals = Some(goals);
                Task::batch(tasks)
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
            Message::GenerateRandomAchievement(ref app_id) => Task::perform(game_view::generate_random_achievement(self.credentials.clone(), app_id.clone()), Message::RandomAchievementGenerated),
            Message::RandomAchievementGenerated(random_achievement) => {
                if let Ok(r) = random_achievement {
                    self.handle_generated_random_achievement(r.0, r.1);
                    Task::perform(Goal::list(self.credentials.clone()), Message::GoalsLoaded)
                }
                else {
                    panic!("{}", random_achievement.unwrap_err().as_str())
                }
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
                let random_game_id = self.owned_games.values().nth(rand::random_range(..self.owned_games.values().len())).unwrap().appid;
                self.view = View::Game(random_game_id).clone();
                Task::perform(game_view::load_game_display(self.credentials.clone(), random_game_id.clone(), self.owned_games.get(&random_game_id).expect("Does not exist").name.clone()), Message::GameLoaded)
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

        let main_view: Element<'_, Message> = match &self.view {
            View::None => column![center_x(text("Welcome to G.A.B.E"))].into(),
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
    owned_games: HashMap<i32, Game>,
}

fn load_data() -> DataLoad {
    let key = env::var("STEAM_API_KEY").expect("You need to set the environment variable STEAM_API_KEY with your API key");
    let steam_id = steam_id_store::get_id().expect("Failed to load steam-id, use the cli and supply a --id first");
    let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    // Sync and update all data
    runtime.block_on(goals::get_and_sync_completed_achievements(&key, &steam_id));
    let owned_games_vec = runtime.block_on(game_fetch::get_owned_games(&key, &steam_id));
    runtime.block_on(goals::refresh_game_completion_cache(&key, &steam_id, &owned_games_vec));
    let owned_games: HashMap<i32, Game> = owned_games_vec.iter().map(|g| (g.appid.clone(), g.clone())).collect::<HashMap<_, _>>();
    DataLoad { 
        credentials: Credentials { key, steam_id },
        owned_games,
    }
}
