use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
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

    // first thing you should do: start consuming incoming messages,
    // otherwise they will back up.
    let client_clone = client.clone();
    let join_handle = tokio::spawn(async move {
        let mut bot = Bot::new(bot_config);
        while let Some(message) = incoming_messages.next().await {
            bot.handle_message(&client_clone, message).await;
        }
    });

    // join a channel
    client.join("nertsal".to_owned());

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

struct Bot {
    save_file: String,
    authorities: HashSet<String>,
    commands: Vec<Command>,
    games: VecDeque<Game>,
    current_game: Option<Game>,
}

impl Bot {
    fn new(config: Config) -> Self {
        let mut authorized_persons = HashSet::new();
        authorized_persons.insert("kuviman".to_owned());
        authorized_persons.insert("Nertsal".to_owned());

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
                        bot.games.push_front(Game {
                            author: sender_name.clone(),
                            name: args,
                        });
                        bot.save_games().unwrap();
                        Some(format!(
                            "@{}, your game has been submitted! There are {} games in the queue.",
                            sender_name,
                            bot.games.len()
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
                    let game = bot.games.pop_back();
                    match game {
                        Some(game) => {
                            bot.save_games().unwrap();
                            let reply = format!("Now playing: {} from @{}", game.name, game.author);
                            bot.current_game = Some(game);
                            Some(reply)
                        }
                        None => {
                            bot.current_game = None;
                            None
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
                        .games
                        .iter()
                        .enumerate()
                        .find(|(_, game)| game.author == sender_name)
                    {
                        reply.push_str(&format!(
                            "@{}, your game is {} in the queue. ",
                            sender_name,
                            pos + 1
                        ))
                    } else if let Some(game) = &bot.current_game {
                        if game.author == sender_name {
                            reply.push_str(&format!(
                                "@{}, we are currently playing your game. ",
                                sender_name
                            ))
                        }
                    }
                    let games_count = bot.games.len();
                    if games_count > 0 {
                        reply.push_str(&format!(
                            "There are {} games in the queue. Next games are: ",
                            games_count
                        ));
                        for i in 0..(3.min(games_count)) {
                            reply.push_str(&format!(
                                "{}. {} from {} ",
                                i + 1,
                                bot.games[i].name,
                                bot.games[i].author
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
                command: |bot, _, _| match &bot.current_game {
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
                command: |bot, _, _| match bot.current_game.take() {
                    Some(game) => {
                        bot.games.push_front(game);
                        Some("Game has been put back in queue".to_owned())
                    }
                    None => Some("Not playing any game at the moment".to_owned()),
                },
            },
            Command {
                name: "stop".to_owned(),
                authorities_required: true,
                command: |bot, _, _| {
                    bot.current_game = None;
                    Some("Current game set to None".to_owned())
                },
            },
        ];

        let mut bot = Self {
            save_file: "nertsalbot.json".to_owned(),
            authorities: config.authorities,
            commands,
            games: VecDeque::new(),
            current_game: None,
        };
        match bot.load_games() {
            Ok(_) => println!("Successfully loaded from json"),
            Err(err) => println!("Error loading from json: {}", err),
        }
        bot
    }
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: ServerMessage,
    ) {
        match message {
            ServerMessage::ClearChat(_) => {}
            ServerMessage::ClearMsg(_) => {}
            ServerMessage::GlobalUserState(_) => {}
            ServerMessage::HostTarget(_) => {}
            ServerMessage::Join(message) => {
                println!("Joined: {}", message.channel_login);
                client
                    .say(message.channel_login, "NertsalBot just joined!".to_owned())
                    .await
                    .unwrap();
            }
            ServerMessage::Notice(_) => {}
            ServerMessage::Part(_) => {}
            ServerMessage::Ping(_) => {}
            ServerMessage::Pong(_) => {}
            ServerMessage::Privmsg(message) => {
                println!(
                    "Got a message in {} from {}: {}",
                    message.channel_login, message.sender.name, message.message_text
                );
                if let Some(reply) = self.check_command(&message) {
                    client.say(message.channel_login, reply).await.unwrap();
                }
            }
            ServerMessage::Reconnect(_) => {}
            ServerMessage::RoomState(_) => {}
            ServerMessage::UserNotice(_) => {}
            ServerMessage::UserState(_) => {}
            ServerMessage::Whisper(_) => {}
            ServerMessage::Generic(_) => {}
            _ => {}
        }
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
        serde_json::to_writer(file, &self.games)?;
        Ok(())
    }
    fn load_games(&mut self) -> Result<(), std::io::Error> {
        let file = std::io::BufReader::new(std::fs::File::open(&self.save_file)?);
        self.games = serde_json::from_reader(file)?;
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
