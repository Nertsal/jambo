use super::*;

pub struct Command {
    pub name: String,
    pub authorities_required: bool,
    pub command: fn(&mut LDBot, String, String) -> Option<String>,
}

impl LDBot {
    pub fn next(&mut self) -> Option<String> {
        self.time_limit = None;
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
                if let Some(_) = self.games_state.current_game.take() {
                    self.save_games().unwrap();
                }
                let reply = format!("The queue is empty. !submit <your game>. ");
                Some(reply)
            }
        }
    }
    pub fn skip(&mut self) -> Option<String> {
        match self.games_state.current_game.take() {
            Some(game) => {
                self.games_state.skipped.push(game);
                self.save_games().unwrap();
                let mut reply = "Game has been skipped. ".to_owned();
                reply.push_str(&self.next().unwrap());
                Some(reply)
            }
            None => Some("Not playing any game at the moment. ".to_owned()),
        }
    }
    fn help_message() -> String {
        "To view current game call !current. To see current queue call !queue. To submit a game call !submit with a link to your game on Ludum Dare website, like so: !submit https://ldjam.com/events/ludum-dare/47/the-island".to_owned()
    }
    pub fn commands() -> Vec<Command> {
        vec![
            Command {
                name: "help".to_owned(),
                authorities_required: false,
                command: |_, _, _| Some(Self::help_message()),
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
                    if !bot.games_state.is_open {
                        Some(
                            "The queue is closed. You can not submit your game at the moment."
                                .to_owned(),
                        )
                    } else if args.is_empty() {
                        Some(Self::help_message())
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
                            .queue()
                            .enumerate()
                            .find(|(_, game)| game.name == args)
                        {
                            return Some(format!("@{}, that game has already been submitted. It is currently {} in the queue.", sender_name, index + 1));
                        }

                        if let Some(_) = bot
                            .games_state
                            .skipped
                            .iter()
                            .find(|game| game.name == args)
                        {
                            return Some(format!("@{}, your game was skipped. You may return to the front of the queue using !return command", sender_name));
                        }

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
                    } else {
                        Some(format!("@{}, that is not a Ludum Dare page", sender_name))
                    }
                },
            },
            Command {
                name: "return".to_owned(),
                authorities_required: false,
                command: |bot, sender_name, _| {
                    if !bot.games_state.is_open {
                        return Some(
                            "The queue is closed. You can not submit your game at the moment."
                                .to_owned(),
                        );
                    }
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
                        bot.save_games().unwrap();
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
                    bot.time_limit = None;
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
                name: "unskip".to_owned(),
                authorities_required: true,
                command: |bot, _, _| {
                    if let Some(skipped) = bot.games_state.skipped.pop() {
                        bot.time_limit = None;
                        let mut reply = String::new();
                        if let Some(current) = bot.games_state.current_game.take() {
                            bot.games_state.returned_queue.push_front(current);
                            reply.push_str("Current game has been put at the front of the queue. ");
                        }
                        reply.push_str(&format!(
                            "Now playing {} from @{}",
                            skipped.name, skipped.author
                        ));
                        bot.games_state.current_game = Some(skipped);
                        Some(reply)
                    } else {
                        Some("No game has been skipped yet".to_owned())
                    }
                },
            },
            Command {
                name: "stop".to_owned(),
                authorities_required: true,
                command: |bot, _, _| {
                    bot.time_limit = None;
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
            Command {
                name: "force".to_owned(),
                authorities_required: true,
                command: |bot, _, _| {
                    if let Some(_) = bot.time_limit.take() {
                        let game = bot.games_state.current_game.as_ref().unwrap();
                        Some(format!("Now playing {} from @{}", game.name, game.author))
                    } else {
                        Some("Not waiting for response at the moment".to_owned())
                    }
                },
            },
            Command {
                name: "close".to_owned(),
                authorities_required: true,
                command: |bot, _, _| {
                    bot.games_state.is_open = false;
                    bot.save_games().unwrap();
                    Some("The queue is now closed".to_owned())
                },
            },
            Command {
                name: "open".to_owned(),
                authorities_required: true,
                command: |bot, _, _| {
                    bot.games_state.is_open = true;
                    bot.save_games().unwrap();
                    Some("The queue is now open".to_owned())
                },
            },
        ]
    }
}
