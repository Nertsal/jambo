use bot_core::prelude::*;
use google_sheets4::Sheets;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

mod commands;
mod google;

use google::*;

#[derive(Serialize, Deserialize)]
pub struct GameJamConfig {
    multiple_submissions: bool,
    queue_mode: bool,
    return_mode: ReturnMode,
    auto_return: bool,
    response_time_limit: Option<u64>,
    link_start: Option<String>,
    allow_direct_link_submit: bool,
    raffle_default_weight: usize,
    google_sheet_config: Option<GoogleSheetConfig>,
}

pub struct GameJamBot {
    cli: CLI,
    config: GameJamConfig,
    commands: Commands<Self, Sender>,
    save_file: String,
    played_games_file: String,
    played_games: Vec<Game>,
    games_state: GamesState,
    time_limit: Option<f32>,
    hub: Option<Sheets>,
    update_sheets: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct Game {
    author: String,
    link: String,
    name: Option<String>,
}

impl Game {
    fn new(author_name: String, link: String) -> Self {
        Self {
            author: author_name,
            name: Game::name_from_link(&link),
            link,
        }
    }

    fn display_name(&self) -> &str {
        self.name.as_ref().unwrap_or(&self.link)
    }

    fn name_from_link(link: &str) -> Option<String> {
        // Ludumdare
        let ludumdare = "https://ldjam.com/events/ludum-dare/";
        if link.starts_with(ludumdare) {
            let mut args = link[ludumdare.len()..].split('/');
            let _ld_number = args.next()?;
            let game_name = args.next()?;
            Some(game_name.to_owned())
        } else {
            None
        }
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} from @{}", self.display_name(), self.author)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
enum GameType {
    Queued,
    Current,
    Skipped,
    Played,
}

#[derive(Serialize, Deserialize)]
struct GamesState {
    is_open: bool,
    current_game: Option<Game>,
    returned_queue: VecDeque<Game>,
    games_queue: VecDeque<Game>,
    skipped: Vec<Game>,
    raffle: Raffle,
}

impl GamesState {
    fn new() -> Self {
        Self {
            is_open: true,
            current_game: None,
            returned_queue: VecDeque::new(),
            games_queue: VecDeque::new(),
            skipped: Vec::new(),
            raffle: Raffle::new(),
        }
    }
    fn queue(&self) -> impl Iterator<Item = &Game> {
        self.returned_queue.iter().chain(self.games_queue.iter())
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
enum ReturnMode {
    Back,
    Front,
}

#[derive(Serialize, Deserialize, Clone)]
struct Raffle {
    viewers_weight: HashMap<String, usize>,
    mode: RaffleMode,
}

#[derive(Serialize, Deserialize, Clone)]
enum RaffleMode {
    Inactive,
    Active { joined: HashMap<String, usize> },
}

impl Raffle {
    fn new() -> Self {
        Self {
            viewers_weight: HashMap::new(),
            mode: RaffleMode::Inactive,
        }
    }
}

impl GameJamBot {
    pub fn name() -> &'static str {
        "GameJamBot"
    }

    pub fn new(cli: &CLI) -> Box<dyn Bot> {
        let config: GameJamConfig = serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open("config/gamejam/gamejam_config.json").unwrap(),
        ))
        .unwrap();

        let mut bot = Self {
            cli: Arc::clone(cli),
            config,
            commands: Self::commands(),
            save_file: "config/gamejam/gamejam_nertsalbot.json".to_owned(),
            played_games_file: "config/gamejam/games_played.json".to_owned(),
            played_games: Vec::new(),
            games_state: GamesState::new(),
            time_limit: None,
            hub: None,
            update_sheets: true,
        };

        if bot.config.google_sheet_config.is_some() {
            let service_key: yup_oauth2::ServiceAccountKey = serde_json::from_reader(
                std::io::BufReader::new(std::fs::File::open("secrets/service_key.json").unwrap()),
            )
            .unwrap();
            let auth = async_std::task::block_on(
                yup_oauth2::ServiceAccountAuthenticator::builder(service_key).build(),
            )
            .unwrap();

            bot.hub = Some(Sheets::new(
                hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
                auth,
            ));
        }

        bot.log(
            LogType::Info,
            &format!("Loading GameJamBot data from {}", &bot.save_file),
        );
        match bot.load_games() {
            Ok(_) => bot.log(
                LogType::Info,
                &format!("Successfully loaded GameJamBot data"),
            ),
            Err(error) => {
                use std::io::ErrorKind;
                match error.kind() {
                    ErrorKind::NotFound => {
                        ("Using default GameJamBot data");
                        bot.save_games().unwrap();
                    }
                    _ => panic!("Error loading GameJamBot data: {}", error),
                }
            }
        }
        match load_from(&bot.played_games_file) {
            Ok(played_games) => bot.played_games = played_games,
            Err(error) => {
                use std::io::ErrorKind;
                match error.kind() {
                    ErrorKind::NotFound => {
                        save_into(&bot.played_games, &bot.played_games_file).unwrap();
                    }
                    _ => panic!("Error loading GameJamBot data: {}", error),
                }
            }
        }
        Box::new(bot)
    }

    fn check_message(&mut self, message: &CommandMessage<Sender>) -> Response {
        if let Some(_) = self.time_limit {
            let game = self.games_state.current_game.as_ref().unwrap();
            if message.sender.name == game.author {
                self.time_limit = None;
                return Some(format!("Now playing {}. ", game));
            }
        }
        if self.config.auto_return {
            return self.return_game(&message.sender.name);
        }
        None
    }

    fn update(&mut self, delta_time: f32) -> Response {
        if let Some(time) = &mut self.time_limit {
            *time -= delta_time;
            if *time <= 0.0 {
                return self.skip(true);
            }
        }
        None
    }

    pub fn save_games(&mut self) -> std::io::Result<()> {
        self.update_sheets = true;
        save_into(&self.games_state, &self.save_file)
    }

    fn load_games(&mut self) -> std::io::Result<()> {
        self.games_state = load_from(&self.save_file)?;
        Ok(())
    }
}

fn save_into<T: Serialize>(value: &T, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    serde_json::to_writer(file, value)?;
    Ok(())
}

fn load_from<T: serde::de::DeserializeOwned>(
    path: impl AsRef<std::path::Path>,
) -> std::io::Result<T> {
    let file = std::io::BufReader::new(std::fs::File::open(path)?);
    Ok(serde_json::from_reader(file)?)
}

#[async_trait]
impl Bot for GameJamBot {
    fn name(&self) -> &str {
        Self::name()
    }

    async fn handle_server_message(&mut self, client: &TwitchClient, message: &ServerMessage) {
        match message {
            ServerMessage::Privmsg(private_message) => {
                let message = private_to_command_message(private_message);
                if let Some(reply) = self.check_message(&message) {
                    self.send_message(client, private_message.channel_login.clone(), reply)
                        .await;
                }
                perform_commands(
                    self,
                    client,
                    private_message.channel_login.clone(),
                    &message,
                )
                .await;
            }
            _ => (),
        };
    }

    async fn update(&mut self, client: &TwitchClient, channel_login: &String, delta_time: f32) {
        if let Some(reply) = self.update(delta_time) {
            self.send_message(client, channel_login.clone(), reply)
                .await;
        }

        if self.update_sheets {
            if self.config.google_sheet_config.is_some() {
                match self.save_sheets().await {
                    Ok(_) => (),
                    Err(err) => self.log(
                        LogType::Error,
                        &format!("Error trying to save queue into google sheets: {}", err),
                    ),
                }
            }
            self.update_sheets = false;
        }
    }

    async fn handle_command_message(
        &mut self,
        client: &TwitchClient,
        channel_login: &String,
        message: &CommandMessage<Sender>,
    ) {
        perform_commands(self, client, channel_login.clone(), &message).await;
    }

    fn get_completion_tree(&self) -> Vec<CompletionNode> {
        commands_to_completion(&self.get_commands().commands)
    }
}
