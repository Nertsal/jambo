use super::*;
use std::collections::VecDeque;

mod commands;

#[derive(Serialize, Deserialize)]
pub struct GameJamConfig {
    response_time_limit: Option<u64>,
    link_start: Option<String>,
    allow_direct_link_submit: bool,
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
        match bot.load_games() {
            Ok(_) => println!("Successfully loaded GameJamBot data"),
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
                return self.skip();
            }
        }
        None
    }
    fn save_played(&self) -> Result<(), std::io::Error> {
        let file = std::io::BufWriter::new(std::fs::File::create(&self.played_games_file)?);
        serde_json::to_writer(file, &self.played_games)?;
        Ok(())
    }
    fn save_games(&self) -> Result<(), std::io::Error> {
        let file = std::io::BufWriter::new(std::fs::File::create(&self.save_file)?);
        serde_json::to_writer(file, &self.games_state)?;
        Ok(())
    }
    fn load_games(&mut self) -> Result<(), std::io::Error> {
        let file = std::io::BufReader::new(std::fs::File::open(&self.save_file)?);
        self.games_state = serde_json::from_reader(file)?;
        let file = std::io::BufReader::new(std::fs::File::open(&self.played_games_file)?);
        self.played_games = serde_json::from_reader(file)?;
        Ok(())
    }
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
}

impl GamesState {
    fn new() -> Self {
        Self {
            is_open: true,
            current_game: None,
            returned_queue: VecDeque::new(),
            games_queue: VecDeque::new(),
            skipped: Vec::new(),
        }
    }
    fn queue(&self) -> impl Iterator<Item = &Game> {
        self.returned_queue.iter().chain(self.games_queue.iter())
    }
}
