use super::*;
use std::collections::VecDeque;

mod commands;

#[derive(Serialize, Deserialize)]
pub struct LDConfig {
    response_time_limit: Option<u64>,
    link_start: Option<String>,
    allow_direct_link_submit: bool,
}

pub struct LDBot {
    channel_login: String,
    save_file: String,
    config: LDConfig,
    commands: BotCommands<Self>,
    games_state: GamesState,
    time_limit: Option<Instant>,
}

impl LDBot {
    pub fn name() -> &'static str {
        "LDBot"
    }
    pub fn new(channel: &String) -> Self {
        let config: LDConfig = serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open("config/ludum_dare/ld-config.json").unwrap(),
        ))
        .unwrap();

        let mut bot = Self {
            channel_login: channel.clone(),
            save_file: "config/ludum_dare/ld-nertsalbot.json".to_owned(),
            config,
            commands: Self::commands(),
            games_state: GamesState::new(),
            time_limit: None,
        };
        println!("Loading LDBot data from {}", &bot.save_file);
        match bot.load_games() {
            Ok(_) => println!("Successfully loaded LDBot data"),
            Err(error) => {
                use std::io::ErrorKind;
                match error.kind() {
                    ErrorKind::NotFound => {
                        println!("Using default LDBot data");
                        bot.save_games().unwrap();
                    }
                    _ => panic!("Error loading LDBot data: {}", error),
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
    fn save_games(&self) -> Result<(), std::io::Error> {
        let file = std::io::BufWriter::new(std::fs::File::create(&self.save_file)?);
        serde_json::to_writer(file, &self.games_state)?;
        Ok(())
    }
    fn load_games(&mut self) -> Result<(), std::io::Error> {
        let file = std::io::BufReader::new(std::fs::File::open(&self.save_file)?);
        self.games_state = serde_json::from_reader(file)?;
        Ok(())
    }
}

#[async_trait]
impl Bot for LDBot {
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
