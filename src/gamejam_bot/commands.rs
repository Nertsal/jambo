use rand::seq::SliceRandom;
use std::sync::Arc;

use super::*;

impl CommandBot<Self> for GameJamBot {
    fn get_commands(&self) -> &BotCommands<Self> {
        &self.commands
    }
}

impl GameJamBot {
    fn set_current(&mut self, game: Game) -> Option<String> {
        let reply = format!("Now playing {} from @{}. ", game.name, game.author);
        self.games_state.current_game = Some(game);
        Some(reply)
    }
    fn remove_game(&mut self, author_name: &str) -> Option<Game> {
        let pos = self
            .games_state
            .returned_queue
            .iter()
            .enumerate()
            .find(|&(_, game)| game.author == author_name)
            .map(|(pos, _)| pos);
        pos.map(|pos| self.games_state.returned_queue.remove(pos))
            .or_else(|| {
                let pos = self
                    .games_state
                    .games_queue
                    .iter()
                    .enumerate()
                    .find(|&(_, game)| game.author == author_name)
                    .map(|(pos, _)| pos);
                pos.map(|pos| self.games_state.games_queue.remove(pos))
            })
            .flatten()
            .or_else(|| {
                let pos = self
                    .games_state
                    .skipped
                    .iter()
                    .enumerate()
                    .find(|&(_, game)| game.author == author_name)
                    .map(|(pos, _)| pos);
                pos.map(|pos| self.games_state.skipped.remove(pos))
            })
    }
    pub fn next(
        &mut self,
        author_name: Option<String>,
        confirmation_required: bool,
    ) -> Option<String> {
        let game = match &author_name {
            Some(author_name) => match self.remove_game(author_name) {
                Some(game) => Ok(game),
                None => Err(format!("Couldn't find a game from {}", author_name)),
            },
            None => match self
                .games_state
                .returned_queue
                .pop_front()
                .or_else(|| self.games_state.games_queue.pop_front())
            {
                Some(game) => Ok(game),
                None => Err(format!("The queue is empty. !submit <your game>. ")),
            },
        };

        if let Some(old_game) = self.games_state.current_game.take() {
            self.played_games.push(old_game);
            save_into(&self.played_games, &self.played_games_file).unwrap();
        }

        self.time_limit = None;
        let reply = match game {
            Ok(game) => {
                let mut reply = None;
                if confirmation_required {
                    if let Some(response_time) = self.config.response_time_limit {
                        self.time_limit = Some(Instant::now());
                        reply = Some(format!(
                            "@{}, we are about to play your game. Please reply in {} seconds. ",
                            game.author, response_time
                        ))
                    }
                }
                let reply = reply.or(self.set_current(game));
                reply
            }
            Err(reply) => Some(reply),
        };

        self.save_games().unwrap();
        reply
    }
    pub fn skip(&mut self) -> Option<String> {
        match self.games_state.current_game.take() {
            Some(game) => {
                self.games_state.skipped.push(game);
                self.save_games().unwrap();
                let reply = "Game has been skipped. ".to_owned();
                Some(reply)
            }
            None => Some("Not playing any game at the moment. ".to_owned()),
        }
    }
    fn help_message() -> String {
        "To view current game call !current. To see current queue call !queue. To submit a game call !submit with a link to your game, like so: !submit https://ldjam.com/events/ludum-dare/47/the-island".to_owned()
    }
    fn check_link(&self, game_link: &str) -> bool {
        if let Some(link_start) = &self.config.link_start {
            game_link.starts_with(link_start)
        } else {
            true
        }
    }
    fn submit(&mut self, game_link: String, sender_name: String) -> Option<String> {
        if !self.games_state.is_open {
            Some("The queue is closed. You can not submit your game at the moment.".to_owned())
        } else if self.check_link(&game_link) {
            if let Some(current_game) = &self.games_state.current_game {
                if current_game.name == game_link {
                    return Some(format!(
                        "@{}, we are playing that game right now!",
                        sender_name
                    ));
                }
            }

            if let Some((index, _)) = self
                .games_state
                .queue()
                .enumerate()
                .find(|(_, game)| game.name == game_link)
            {
                return Some(format!(
                    "@{}, that game has already been submitted. It is currently {} in the queue.",
                    sender_name,
                    index + 1
                ));
            }

            if let Some(_) = self
                .games_state
                .skipped
                .iter()
                .find(|game| game.name == game_link)
            {
                return Some(format!("@{}, your game was skipped. You may return to the front of the queue using !return command", sender_name));
            }

            if let Some(_) = self.played_games.iter().find(|game| game.name == game_link) {
                return Some(format!(
                    "@{}, we have already played that game.",
                    sender_name
                ));
            }

            self.games_state.games_queue.push_back(Game {
                author: sender_name.clone(),
                name: game_link,
            });
            self.save_games().unwrap();
            Some(format!(
                "@{}, your game has been submitted! There are {} games in the queue.",
                sender_name,
                self.games_state.games_queue.len()
            ))
        } else {
            Some(format!("@{}, that link can not be submitted", sender_name))
        }
    }
    fn raffle_start(&mut self) -> Option<String> {
        self.games_state.raffle = Raffle::Active {
            joined: HashSet::new(),
        };
        Some(format!(
            "The raffle has started! Type !join to join the raffle."
        ))
    }
    fn raffle_finish(&mut self) -> Option<String> {
        let raffle = std::mem::replace(&mut self.games_state.raffle, Raffle::Inactive);
        match raffle {
            Raffle::Active { joined } => match (joined.into_iter().collect::<Vec<String>>())
                .choose(&mut rand::thread_rng())
            {
                Some(sender_name) => match self.remove_game(sender_name) {
                    Some(game) => self.set_current(game),
                    None => Some(format!("{} has won the raffle!", sender_name)),
                },
                None => Some(format!("Noone has joined the raffle :(")),
            },
            _ => Some(format!("The raffle should be started first: !raffle")),
        }
    }
    fn raffle_join(&mut self, sender_name: String) -> Option<String> {
        match &mut self.games_state.raffle {
            Raffle::Active { joined } => {
                joined.insert(sender_name);
            }
            _ => (),
        }
        None
    }
    fn raffle_undo(&mut self) -> Option<String> {
        self.games_state.raffle = Raffle::Inactive;
        Some(format!("Raffle is now inactive"))
    }
    pub fn commands() -> BotCommands<Self> {
        BotCommands {
            commands: vec![
                CommandNode::ArgumentNode {
                    argument_type: ArgumentType::Word,
                    child_node: Box::new(CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Any,
                        command: Arc::new(|bot, sender_name, mut args| {
                            let game_link = args.remove(0);
                            if bot.config.allow_direct_link_submit
                                && bot.config.link_start.is_some()
                                && bot.check_link(&game_link)
                            {
                                return bot.submit(game_link, sender_name);
                            }
                            None
                        }),
                    }),
                },
                CommandNode::LiteralNode {
                    literal: "!help".to_owned(),
                    child_nodes: vec![CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Any,
                        command: Arc::new(|_, _, _| Some(Self::help_message())),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "!submit".to_owned(),
                    child_nodes: vec![
                        CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Any,
                            command: Arc::new(|_, _, _| Some(Self::help_message())),
                        },
                        CommandNode::ArgumentNode {
                            argument_type: ArgumentType::Word,
                            child_node: Box::new(CommandNode::FinalNode {
                                authority_level: AuthorityLevel::Any,
                                command: Arc::new(|bot, sender_name, mut args| {
                                    let game_link = args.remove(0);
                                    bot.submit(game_link, sender_name)
                                }),
                            }),
                        },
                    ],
                },
                CommandNode::LiteralNode {
                    literal: "!return".to_owned(),
                    child_nodes: vec![CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Any,
                        command: Arc::new(|bot, sender_name, _| {
                            if !bot.games_state.is_open {
                                return Some(
                                            "The queue is closed. You can not submit your game at the moment."
                                                .to_owned(),
                                        );
                            }
                            let reply = if let Some(game) = bot
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
                                Some(format!(
                                    "@{}, your game was returned to the front of the queue",
                                    sender_name
                                ))
                            } else {
                                None
                            };
                            reply
                        }),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "!next".to_owned(),
                    child_nodes: vec![
                        CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Broadcaster,
                            command: Arc::new(|bot, _, _| bot.next(None, true)),
                        },
                        CommandNode::ArgumentNode {
                            argument_type: ArgumentType::Line,
                            child_node: Box::new(CommandNode::FinalNode {
                                authority_level: AuthorityLevel::Broadcaster,
                                command: Arc::new(|bot, _, mut args| {
                                    let author_name = args.remove(0);
                                    bot.next(Some(author_name), false)
                                }),
                            }),
                        },
                    ],
                },
                CommandNode::LiteralNode {
                    literal: "!cancel".to_owned(),
                    child_nodes: vec![
                        CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Any,
                            command: Arc::new(|bot, sender_name, _| {
                                match bot.remove_game(&sender_name) {
                                    Some(_) => {
                                        let reply = format!(
                                            "{}'s game has been remove from the queue",
                                            sender_name
                                        );
                                        Some(reply)
                                    }
                                    None => {
                                        let reply =
                                            format!("Couldn't find a game from {}", sender_name);
                                        Some(reply)
                                    }
                                }
                            }),
                        },
                        CommandNode::ArgumentNode {
                            argument_type: ArgumentType::Line,
                            child_node: Box::new(CommandNode::FinalNode {
                                authority_level: AuthorityLevel::Moderator,
                                command: Arc::new(|bot, _, mut args| {
                                    let author_name = args.remove(0);
                                    match bot.remove_game(&author_name) {
                                        Some(_) => {
                                            let reply = format!(
                                                "{}'s game has been remove from the queue",
                                                author_name
                                            );
                                            Some(reply)
                                        }
                                        None => {
                                            let reply = format!(
                                                "Couldn't find a game from {}",
                                                author_name
                                            );
                                            Some(reply)
                                        }
                                    }
                                }),
                            }),
                        },
                    ],
                },
                CommandNode::LiteralNode {
                    literal: "!queue".to_owned(),
                    child_nodes: vec![CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Any,
                        command: Arc::new(|bot, sender_name, _| {
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
                        }),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "!current".to_owned(),
                    child_nodes: vec![CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Any,
                        command: Arc::new(|bot, _, _| match &bot.games_state.current_game {
                            Some(game) => Some(format!(
                                "Current game is: {} from {}",
                                game.name, game.author
                            )),
                            None => Some("Not playing any game at the moment".to_owned()),
                        }),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "!skip".to_owned(),
                    child_nodes: vec![CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Moderator,
                        command: Arc::new(|bot, _, _| bot.skip()),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "!unskip".to_owned(),
                    child_nodes: vec![CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Moderator,
                        command: Arc::new(|bot, _, _| {
                            if let Some(skipped) = bot.games_state.skipped.pop() {
                                bot.time_limit = None;
                                let mut reply = String::new();
                                if let Some(current) = bot.games_state.current_game.take() {
                                    bot.games_state.returned_queue.push_front(current);
                                    reply.push_str(
                                        "Current game has been put at the front of the queue. ",
                                    );
                                }
                                match bot.set_current(skipped) {
                                    Some(set_reply) => reply.push_str(&set_reply),
                                    None => (),
                                }
                                Some(reply)
                            } else {
                                Some("No game has been skipped yet".to_owned())
                            }
                        }),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "!stop".to_owned(),
                    child_nodes: vec![CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Moderator,
                        command: Arc::new(|bot, _, _| {
                            bot.time_limit = None;
                            bot.games_state.current_game = None;
                            bot.save_games().unwrap();
                            Some("Current game set to None".to_owned())
                        }),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "!clear".to_owned(),
                    child_nodes: vec![CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Moderator,
                        command: Arc::new(|bot, _, _| {
                            bot.games_state.games_queue.clear();
                            bot.save_games().unwrap();
                            Some("The queue has been cleared".to_owned())
                        }),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "!force".to_owned(),
                    child_nodes: vec![CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Moderator,
                        command: Arc::new(|bot, _, _| {
                            if let Some(_) = bot.time_limit.take() {
                                let game = bot.games_state.current_game.as_ref().unwrap();
                                Some(format!("Now playing {} from @{}", game.name, game.author))
                            } else {
                                Some("Not waiting for response at the moment".to_owned())
                            }
                        }),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "!close".to_owned(),
                    child_nodes: vec![CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Moderator,
                        command: Arc::new(|bot, _, _| {
                            bot.games_state.is_open = false;
                            bot.save_games().unwrap();
                            Some("The queue is now closed".to_owned())
                        }),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "!open".to_owned(),
                    child_nodes: vec![CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Moderator,
                        command: Arc::new(|bot, _, _| {
                            bot.games_state.is_open = true;
                            bot.save_games().unwrap();
                            Some("The queue is now open".to_owned())
                        }),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "!raffle".to_owned(),
                    child_nodes: vec![
                        CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Broadcaster,
                            command: Arc::new(|bot, _, _| bot.raffle_start()),
                        },
                        CommandNode::LiteralNode {
                            literal: "finish".to_owned(),
                            child_nodes: vec![CommandNode::FinalNode {
                                authority_level: AuthorityLevel::Broadcaster,
                                command: Arc::new(|bot, _, _| bot.raffle_finish()),
                            }],
                        },
                        CommandNode::LiteralNode {
                            literal: "undo".to_owned(),
                            child_nodes: vec![CommandNode::FinalNode {
                                authority_level: AuthorityLevel::Broadcaster,
                                command: Arc::new(|bot, _, _| bot.raffle_undo()),
                            }],
                        },
                    ],
                },
                CommandNode::LiteralNode {
                    literal: "!join".to_owned(),
                    child_nodes: vec![CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Any,
                        command: Arc::new(|bot, sender_name, _| bot.raffle_join(sender_name)),
                    }],
                },
            ],
        }
    }
}
