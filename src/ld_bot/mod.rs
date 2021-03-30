use super::*;
use std::collections::{HashSet, VecDeque};

mod commands;

use commands::*;

#[derive(Serialize, Deserialize)]
struct LDConfig {
    authorities: HashSet<String>,
    response_time_limit: Option<u64>,
}

pub struct LDBot {
    channel_login: String,
    save_file: String,
    response_time_limit: Option<u64>,
    authorities: HashSet<String>,
    commands: Vec<Command>,
    games_state: GamesState,
    time_limit: Option<Instant>,
}

impl LDBot {
    pub fn new(channel: &String) -> Self {
        let config: LDConfig = serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open("ld-config.json").unwrap(),
        ))
        .unwrap();

        let mut bot = Self {
            channel_login: channel.clone(),
            save_file: "ld-nertsalbot.json".to_owned(),
            response_time_limit: config.response_time_limit,
            authorities: config.authorities.clone(),
            commands: Self::commands(),
            games_state: GamesState::new(),
            time_limit: None,
        };
        println!("Loading data from {}", &bot.save_file);
        match bot.load_games() {
            Ok(_) => println!("Successfully loaded from json"),
            Err(err) => {
                panic!("Error loading from json: {}", err);
            }
        }
        bot
    }
    fn check_command(&mut self, message: &PrivmsgMessage) -> Option<String> {
        let mut message_text = message.message_text.clone();
        let sender_name = message.sender.name.clone();

        if let Some(_) = self.time_limit {
            let game = self.games_state.current_game.as_ref().unwrap();
            if message.sender.name == game.author {
                self.time_limit = None;
                let mut reply = format!("Now playing {} from @{}. ", game.name, game.author);
                if let Some(command_reply) = self.check_command(message) {
                    reply.push_str(&command_reply);
                }
                return Some(reply);
            }
        }

        match message_text.remove(0) {
            '!' => {
                let mut args = message_text.split_whitespace();
                if let Some(command) = args.next() {
                    if let Some(command) = self.commands.iter().find_map(|com| {
                        if com.name == command {
                            if com.authorities_required
                                && !self.authorities.contains(&message.sender.login)
                            {
                                return None;
                            }
                            Some(com.command)
                        } else {
                            None
                        }
                    }) {
                        return command(self, sender_name, args.collect());
                    }
                }
                None
            }
            _ => None,
        }
    }
    fn update(&mut self) -> Option<String> {
        if let Some(time) = self.time_limit {
            if time.elapsed().as_secs() >= self.response_time_limit.unwrap() {
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
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &ServerMessage,
    ) {
        if let Some(reply) = self.update() {
            client.say(self.channel_login.clone(), reply).await.unwrap();
        }
        match message {
            ServerMessage::Join(message) => {
                println!("Joined: {}", message.channel_login);
            }
            ServerMessage::Privmsg(message) => {
                println!(
                    "Got a message in {} from {}: {}",
                    message.channel_login, message.sender.name, message.message_text
                );
                if let Some(reply) = self.check_command(&message) {
                    client
                        .say(message.channel_login.clone(), reply)
                        .await
                        .unwrap();
                }
            }
            ServerMessage::Notice(message) => {
                if message.message_text == "Login authentication failed" {
                    panic!("Login authentication failed.");
                }
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
