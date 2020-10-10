use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
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
        match message {
            ServerMessage::Join(message) => {
                println!("Joined: {}", message.channel_login);
            }
            ServerMessage::Privmsg(message) => {
                println!(
                    "Got a message in {} from {}: {}",
                    message.channel_login, message.sender.name, message.message_text
                );
                if let Some(reply) = self
                    .bots
                    .get_mut(&message.channel_login)
                    .unwrap()
                    .check_command(&message)
                {
                    client.say(message.channel_login, reply).await.unwrap();
                }
            }
            _ => (),
        }
    }
}

struct Bot {
    save_file: String,
    authorities: HashSet<String>,
    commands: Vec<Command>,
    games_state: GamesState,
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
                name: "submit".to_owned(),
                authorities_required: false,
                command: |bot, sender_name, args| {
                    if args.is_empty() {
                        Some(help_message())
                    } else if args.starts_with("https://ldjam.com/events/ludum-dare/") {
                        bot.games_state.games_queue.push_front(Game {
                            author: sender_name.clone(),
                            name: args,
                        });
                        bot.save_games().unwrap();
                        Some(format!(
                            "@{}, your game has been submitted! There are {} games in the queue.",
                            sender_name,
                            bot.games_state.games_queue.len()
                        ))
                    } else {
                        Some(format!("@{}, that is not a Ludum Dare page", sender_name))
                    }
                },
            },
            Command {
                name: "next".to_owned(),
                authorities_required: true,
                command: |bot, _, _| {
                    let game = bot.games_state.games_queue.pop_back();
                    match game {
                        Some(game) => {
                            let reply = format!("Now playing: {} from @{}", game.name, game.author);
                            bot.games_state.current_game = Some(game);
                            bot.save_games().unwrap();
                            Some(reply)
                        }
                        None => {
                            bot.games_state.current_game = None;
                            let reply = format!("The queue is empty. !submit <your game>");
                            Some(reply)
                        }
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
                        .games_queue
                        .iter()
                        .enumerate()
                        .find(|(_, game)| game.author == sender_name)
                    {
                        reply.push_str(&format!(
                            "@{}, your game is {} in the queue. ",
                            sender_name,
                            pos + 1
                        ))
                    } else if let Some(game) = &bot.games_state.current_game {
                        if game.author == sender_name {
                            reply.push_str(&format!(
                                "@{}, we are currently playing your game. ",
                                sender_name
                            ))
                        }
                    }
                    let games_count = bot.games_state.games_queue.len();
                    if games_count > 0 {
                        reply.push_str(&format!(
                            "There are {} games in the queue. Next games are: ",
                            games_count
                        ));
                        for i in 0..(3.min(games_count)) {
                            reply.push_str(&format!(
                                "{}. {} from {} ",
                                i + 1,
                                bot.games_state.games_queue[i].name,
                                bot.games_state.games_queue[i].author
                            ))
                        }
                    } else {
                        reply.push_str("No games in queue");
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
                name: "current-queue".to_owned(),
                authorities_required: true,
                command: |bot, _, _| match bot.games_state.current_game.take() {
                    Some(game) => {
                        bot.games_state.games_queue.push_front(game);
                        bot.save_games().unwrap();
                        Some("Game has been put back in queue".to_owned())
                    }
                    None => Some("Not playing any game at the moment".to_owned()),
                },
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
        ];

        let mut bot = Self {
            save_file,
            authorities: config.authorities.clone(),
            commands,
            games_state: GamesState::new(),
        };
        println!("Loading data from {}", &bot.save_file);
        match bot.load_games() {
            Ok(_) => println!("Successfully loaded from json"),
            Err(err) => println!("Error loading from json: {}", err),
        }
        bot
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
}

fn help_message() -> String {
    "To view current game call !current. To see current queue call !queue. To submit a game call !submit with a link to your game on Ludum Dare website, like so: !submit https://ldjam.com/events/ludum-dare/47/the-island".to_owned()
}

struct Command {
    name: String,
    authorities_required: bool,
    command: fn(&mut Bot, String, String) -> Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Game {
    author: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct GamesState {
    current_game: Option<Game>,
    games_queue: VecDeque<Game>,
}

impl GamesState {
    fn new() -> Self {
        Self {
            current_game: None,
            games_queue: VecDeque::new(),
        }
    }
}
