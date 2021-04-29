use super::*;
use std::collections::VecDeque;

mod commands;

#[derive(Serialize, Deserialize)]
pub struct GameJamConfig {
    enable_queue_command: bool,
    response_time_limit: Option<u64>,
    link_start: Option<String>,
    allow_direct_link_submit: bool,
    raffle_default_weight: usize,
}

pub struct GameJamBot {
    channel_login: String,
    config: GameJamConfig,
    commands: BotCommands<Self>,
    save_file: String,
    played_games_file: String,
    played_games: Vec<Game>,
    games_state: GamesState,
    time_limit: Option<Instant>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Game {
    author: String,
    name: String,
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
    pub fn new(channel: &String) -> Self {
        let config: GameJamConfig = serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open("config/gamejam/gamejam-config.json").unwrap(),
        ))
        .unwrap();

        let mut bot = Self {
            channel_login: channel.clone(),
            config,
            commands: Self::commands(),
            save_file: "config/gamejam/gamejam-nertsalbot.json".to_owned(),
            played_games_file: "config/gamejam/games-played.json".to_owned(),
            played_games: Vec::new(),
            games_state: GamesState::new(),
            time_limit: None,
        };
        println!("Loading GameJamBot data from {}", &bot.save_file);
        match load_from(&bot.save_file) {
            Ok(games_state) => {
                bot.games_state = games_state;
                println!("Successfully loaded GameJamBot data")
            }
            Err(error) => {
                use std::io::ErrorKind;
                match error.kind() {
                    ErrorKind::NotFound => {
                        println!("Using default GameJamBot data");
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
        bot
    }
    fn check_message(&mut self, message: &PrivmsgMessage) -> Option<String> {
        if let Some(_) = self.time_limit {
            let game = self.games_state.current_game.as_ref().unwrap();
            if message.sender.name == game.author {
                self.time_limit = None;
                let reply = format!("Now playing {} from @{}. ", game.name, game.author);
                return Some(reply);
            }
        }
        None
    }
    fn update(&mut self) -> Option<String> {
        if let Some(time) = self.time_limit {
            if time.elapsed().as_secs() >= self.config.response_time_limit.unwrap() {
                self.time_limit = None;
                return self.skip(true);
            }
        }
        None
    }
    fn save_games(&self) -> Result<(), std::io::Error> {
        save_into(&self.games_state, &self.save_file)
    }
}

fn save_into<T: Serialize>(
    value: &T,
    path: impl AsRef<std::path::Path>,
) -> Result<(), std::io::Error> {
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    serde_json::to_writer(file, value)?;
    Ok(())
}
fn load_from<T: serde::de::DeserializeOwned>(
    path: impl AsRef<std::path::Path>,
) -> Result<T, std::io::Error> {
    let file = std::io::BufReader::new(std::fs::File::open(path)?);
    Ok(serde_json::from_reader(file)?)
}

#[async_trait]
impl Bot for GameJamBot {
    fn name(&self) -> &str {
        Self::name()
    }
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &ServerMessage,
    ) {
        if let Some(reply) = self.update() {
            send_message(client, self.channel_login.clone(), reply).await;
        }
        match message {
            ServerMessage::Privmsg(message) => {
                if let Some(reply) = self.check_message(message) {
                    send_message(client, self.channel_login.clone(), reply).await;
                }
                check_command(self, client, self.channel_login.clone(), message).await;
            }
            _ => (),
        };
    }
}
