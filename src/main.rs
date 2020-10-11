use futures::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::{PrivmsgMessage, ServerMessage};
use twitch_irc::{ClientConfig, TCPTransport, TwitchIRCClient};

#[tokio::main]
async fn main() {
    let bot_config: Config = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("config.json").unwrap(),
    ))
    .unwrap();

    let config = ClientConfig::new_simple(StaticLoginCredentials::new(
        bot_config.login_name.clone(),
        Some(bot_config.oauth_token.clone()),
    ));
    let (mut incoming_messages, client) =
        TwitchIRCClient::<TCPTransport, StaticLoginCredentials>::new(config);

    let channels = bot_config.channels.clone();
    let mut channels_bot = ChannelsBot {
        bots: {
            let mut map = HashMap::new();
            for channel in &channels {
                let mut save_file = channel.clone();
                save_file.push_str("-nertsalbot.json");
                map.insert(channel.clone(), Bot::new(&bot_config, save_file));
            }
            map
        },
    };

    // first thing you should do: start consuming incoming messages,
    // otherwise they will back up.
    let client_clone = client.clone();
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.next().await {
            channels_bot.handle_message(&client_clone, message).await;
        }
    });

    // join a channel
    for channel in channels {
        client.join(channel);
    }

    // keep the tokio executor alive.
    // If you return instead of waiting the background task will exit.
    join_handle.await.unwrap();
}

#[derive(Serialize, Deserialize)]
struct Config {
    login_name: String,
    oauth_token: String,
    channels: Vec<String>,
    authorities: HashSet<String>,
    response_time_limit: Option<u64>,
}

struct ChannelsBot {
    bots: HashMap<String, Bot>,
}

impl ChannelsBot {
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: ServerMessage,
    ) {
        for (channel, bot) in &mut self.bots {
            if let Some(reply) = bot.update() {
                client.say(channel.clone(), reply).await.unwrap();
            }
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
                let bot = self.bots.get_mut(&message.channel_login).unwrap();
                if let Some(reply) = bot.check_command(&message) {
                    client
                        .say(message.channel_login.clone(), reply)
                        .await
                        .unwrap();
                }
                if let Some(reply) = bot.check_message(&message) {
                    client.say(message.channel_login, reply).await.unwrap();
                }
            }
            _ => (),
        }
    }
}

struct Bot {
    save_file: String,
    response_time_limit: Option<u64>,
    authorities: HashSet<String>,
    commands: Vec<Command>,
    games_state: GamesState,
    time_limit: Option<Instant>,
}

impl Bot {
    fn new(config: &Config, save_file: String) -> Self {
        let commands = vec![
            Command {
                name: "help".to_owned(),
                authorities_required: false,
                command: |_, _, _| Some(help_message()),
            },
            Command {
                name: "game".to_owned(),
                authorities_required: false,
                command: |_, _, _| {
                    Some("Try our multiplayer sandbox game: https://ldjam.com/events/ludum-dare/47/the-island".to_owned())
                },
            },
            Command {
                name: "submit".to_owned(),
                authorities_required: false,
                command: |bot, sender_name, args| {
                    if args.is_empty() {
                        Some(help_message())
                    } else if args.starts_with("https://ldjam.com/events/ludum-dare/") {
                        if let Some(current_game) = &bot.games_state.current_game {
                            if current_game.name == args {
                                return Some(format!(
                                    "@{}, we are playing that game right now!",
                                    sender_name
                                ));
                            }
                        }

                        if let Some((index, _)) = bot
                            .games_state
                            .games_queue
                            .iter()
                            .enumerate()
                            .find(|(_, game)| game.name == args)
                        {
                            Some(format!("@{}, that game has already been submitted. It is currently {} in the queue.", sender_name, index + 1))
                        } else {
                            bot.games_state.games_queue.push_back(Game {
                                author: sender_name.clone(),
                                name: args,
                            });
                            bot.save_games().unwrap();
                            Some(format!(
                                "@{}, your game has been submitted! There are {} games in the queue.",
                                sender_name,
                                bot.games_state.games_queue.len()
                            ))
                        }
                    } else {
                        Some(format!("@{}, that is not a Ludum Dare page", sender_name))
                    }
                },
            },
            Command {
                name: "return".to_owned(),
                authorities_required: false,
                command: |bot, sender_name, _| {
                    let mut reply = String::new();
                    if let Some(game) = bot
                        .games_state
                        .skipped
                        .iter()
                        .find(|game| game.author == sender_name)
                    {
                        bot.games_state.returned_queue.push_back(game.clone());
                        bot.games_state
                            .skipped
                            .retain(|game| game.author != sender_name);
                        reply.push_str(&format!(
                            "@{}, your game was returned to the front of the queue",
                            sender_name
                        ));
                    } else {
                        reply.push_str(&format!(
                            "@{}, you have caused stack underflow exception, return failed.",
                            sender_name,
                        ));
                    }
                    Some(reply)
                },
            },
            Command {
                name: "next".to_owned(),
                authorities_required: true,
                command: |bot, _, _| bot.next(),
            },
            Command {
                name: "random".to_owned(),
                authorities_required: true,
                command: |bot, _, _| {
                    let skipped_count = bot.games_state.skipped.len();
                    if skipped_count > 0 {
                        let game = bot
                            .games_state
                            .skipped
                            .remove(rand::thread_rng().gen_range(0, skipped_count));
                        let reply = format!("Now playing: {} from @{}", game.name, game.author);
                        bot.games_state.current_game = Some(game);
                        bot.save_games().unwrap();
                        Some(reply)
                    } else {
                        bot.games_state.current_game = None;
                        let reply = format!("No games have been skipped yet");
                        Some(reply)
                    }
                },
            },
            Command {
                name: "queue".to_owned(),
                authorities_required: false,
                command: |bot, sender_name, _| {
                    let mut reply = String::new();
                    if let Some((pos, _)) = bot
                        .games_state
                        .queue()
                        .enumerate()
                        .find(|(_, game)| game.author == sender_name)
                    {
                        reply.push_str(&format!(
                            "@{}, your game is {} in the queue. ",
                            sender_name,
                            pos + 1
                        ))
                    } else if bot
                        .games_state
                        .skipped
                        .iter()
                        .any(|game| game.author == sender_name)
                    {
                        reply.push_str(&format!("@{}, your game was skipped. You may return to the front of the queue using !return command. ", sender_name))
                    } else if let Some(game) = &bot.games_state.current_game {
                        if game.author == sender_name {
                            reply.push_str(&format!(
                                "@{}, we are currently playing your game. ",
                                sender_name
                            ))
                        }
                    }
                    let mut queue = bot.games_state.queue();
                    let mut empty = true;
                    for i in 0..3 {
                        if let Some(game) = queue.next() {
                            if empty {
                                reply.push_str("Queued games: ");
                                empty = false;
                            }
                            reply.push_str(&format!(
                                "{}) {} from {}. ",
                                i + 1,
                                game.name,
                                game.author
                            ));
                        }
                    }
                    if empty {
                        reply.push_str("The queue is empty");
                    } else {
                        let left_count = queue.count();
                        if left_count != 0 {
                            reply.push_str(&format!("And {} more", left_count));
                        }
                    }
                    Some(reply)
                },
            },
            Command {
                name: "current".to_owned(),
                authorities_required: false,
                command: |bot, _, _| match &bot.games_state.current_game {
                    Some(game) => Some(format!(
                        "Current game is: {} from {}",
                        game.name, game.author
                    )),
                    None => Some("Not playing any game at the moment".to_owned()),
                },
            },
            Command {
                name: "skip".to_owned(),
                authorities_required: true,
                command: |bot, _, _| bot.skip(),
            },
            Command {
                name: "stop".to_owned(),
                authorities_required: true,
                command: |bot, _, _| {
                    bot.games_state.current_game = None;
                    bot.save_games().unwrap();
                    Some("Current game set to None".to_owned())
                },
            },
            Command {
                name: "clear".to_owned(),
                authorities_required: true,
                command: |bot, _, _| {
                    bot.games_state.games_queue.clear();
                    bot.save_games().unwrap();
                    Some("The queue has been cleared".to_owned())
                },
            },
        ];

        let mut bot = Self {
            save_file,
            response_time_limit: config.response_time_limit,
            authorities: config.authorities.clone(),
            commands,
            games_state: GamesState::new(),
            time_limit: None,
        };
        println!("Loading data from {}", &bot.save_file);
        match bot.load_games() {
            Ok(_) => println!("Successfully loaded from json"),
            Err(err) => println!("Error loading from json: {}", err),
        }
        bot
    }
    fn check_message(&mut self, message: &PrivmsgMessage) -> Option<String> {
        if let Some(_) = self.time_limit {
            let game = self.games_state.current_game.as_ref().unwrap();
            if message.sender.name == game.author {
                return Some(format!("Now playing {} from @{}", game.name, game.author));
            }
        }
        None
    }
    fn check_command(&mut self, message: &PrivmsgMessage) -> Option<String> {
        let mut message_text = message.message_text.clone();
        let sender_name = message.sender.name.clone();

        match message_text.remove(0) {
            '!' => {
                let mut args = message_text.split_whitespace();
                if let Some(command) = args.next() {
                    if let Some(command) = self.commands.iter().find_map(|com| {
                        if com.name == command {
                            if com.authorities_required && !self.authorities.contains(&sender_name)
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
    fn update(&mut self) -> Option<String> {
        if let Some(time) = self.time_limit {
            if time.elapsed().as_secs() >= self.response_time_limit.unwrap() {
                self.time_limit = None;
                return self.skip();
            }
        }
        None
    }
    fn next(&mut self) -> Option<String> {
        let game = self
            .games_state
            .returned_queue
            .pop_front()
            .or_else(|| self.games_state.games_queue.pop_front());
        match game {
            Some(game) => {
                let reply = if let Some(response_time) = self.response_time_limit {
                    self.time_limit = Some(Instant::now());
                    format!(
                        "@{}, we are about to play your game. Please reply in {} seconds. ",
                        game.author, response_time
                    )
                } else {
                    format!("Now playing {} from @{}. ", game.name, game.author)
                };
                self.games_state.current_game = Some(game);
                self.save_games().unwrap();
                Some(reply)
            }
            None => {
                self.games_state.current_game = None;
                let reply = format!("The queue is empty. !submit <your game>. ");
                Some(reply)
            }
        }
    }
    fn skip(&mut self) -> Option<String> {
        match self.games_state.current_game.take() {
            Some(game) => {
                self.games_state.skipped.push(game);
                let mut reply = "Game has been skipped. ".to_owned();
                reply.push_str(&self.next().unwrap());
                Some(reply)
            }
            None => Some("Not playing any game at the moment. ".to_owned()),
        }
    }
}

fn help_message() -> String {
    "To view current game call !current. To see current queue call !queue. To submit a game call !submit with a link to your game on Ludum Dare website, like so: !submit https://ldjam.com/events/ludum-dare/47/the-island".to_owned()
}

struct Command {
    name: String,
    authorities_required: bool,
    command: fn(&mut Bot, String, String) -> Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Game {
    author: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct GamesState {
    current_game: Option<Game>,
    returned_queue: VecDeque<Game>,
    games_queue: VecDeque<Game>,
    skipped: Vec<Game>,
}

impl GamesState {
    fn new() -> Self {
        Self {
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
