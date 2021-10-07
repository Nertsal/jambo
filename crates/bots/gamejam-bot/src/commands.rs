use rand::seq::SliceRandom;
use std::sync::Arc;

use super::*;

impl CommandBot<Self, Sender> for GameJamBot {
    fn get_commands(&self) -> &Commands<Self, Sender> {
        &self.commands
    }

    fn get_cli(&self) -> &CLI {
        &self.cli
    }
}

impl GameJamBot {
    pub fn set_current(&mut self, game: Option<Game>) -> Response {
        let state = std::mem::take(&mut self.save_state.current_state);
        match state {
            GameJamState::Playing { game } | GameJamState::Waiting { game, .. } => {
                self.played_games.push(game.clone());
                save_into(&self.played_games, PLAYED_GAMES_FILE).unwrap();
            }
            _ => (),
        }

        let reply = match game {
            Some(game) => {
                self.update_status(&format!("Playing {}", game.to_string_name(true)));
                let reply = format!("Now playing {}. ", game.to_string_link(true));
                self.save_state.raffle_viewer_weights.remove(&game.author);
                self.save_state.current_state = GameJamState::Playing { game };
                Some(reply)
            }
            None => {
                self.update_status("Not playing a game");
                self.save_state.current_state = GameJamState::Idle;
                None
            }
        };

        self.save_games().unwrap();
        reply
    }

    fn find_game(&self, predicate: impl Fn(&Game) -> bool) -> Option<(&Game, GameType)> {
        // Check current
        if let GameJamState::Playing { game } | GameJamState::Waiting { game, .. } =
            &self.save_state.current_state
        {
            if predicate(game) {
                return Some((game, GameType::Current));
            }
        }

        // Look in the queue
        let game = self.save_state.queue().find(|game| predicate(game));
        if let Some(game) = game {
            return Some((game, GameType::Queued));
        }

        // Look in the skipped list
        let game = self.save_state.skipped.iter().find(|game| predicate(game));
        if let Some(game) = game {
            return Some((game, GameType::Skipped));
        }

        // Look in the played list
        let game = self.played_games.iter().find(|game| predicate(game));
        if let Some(game) = game {
            return Some((game, GameType::Played));
        }

        None
    }

    fn remove_game(&mut self, author_name: &str) -> Option<Game> {
        self.save_state.current_state = GameJamState::Idle;

        let pos = self
            .save_state
            .returned_queue
            .iter()
            .enumerate()
            .find(|&(_, game)| game.author == author_name)
            .map(|(pos, _)| pos);
        pos.map(|pos| self.save_state.returned_queue.remove(pos))
            .or_else(|| {
                let pos = self
                    .save_state
                    .games_queue
                    .iter()
                    .enumerate()
                    .find(|&(_, game)| game.author == author_name)
                    .map(|(pos, _)| pos);
                pos.map(|pos| self.save_state.games_queue.remove(pos))
            })
            .flatten()
            .or_else(|| {
                let pos = self
                    .save_state
                    .skipped
                    .iter()
                    .enumerate()
                    .find(|&(_, game)| game.author == author_name)
                    .map(|(pos, _)| pos);
                pos.map(|pos| self.save_state.skipped.remove(pos))
            })
    }

    fn remove_game_response(&mut self, author_name: &str) -> Response {
        match self.remove_game(author_name) {
            Some(_) => {
                let reply = format!("{}'s game has been removed from the queue", author_name);
                Some(reply)
            }
            None => {
                let reply = format!("Couldn't find a game from {}", author_name);
                Some(reply)
            }
        }
    }

    pub fn next(&mut self, author_name: Option<String>, confirmation_required: bool) -> Response {
        let game = match &author_name {
            Some(author_name) => match self.remove_game(author_name) {
                Some(game) => Ok(game),
                None => Err(format!("Couldn't find a game from {}", author_name)),
            },
            None => match self
                .save_state
                .returned_queue
                .pop_front()
                .or_else(|| self.save_state.games_queue.pop_front())
            {
                Some(game) => Ok(game),
                None => Err(format!("The queue is empty. !submit <your game>. ")),
            },
        };

        let reply = match game {
            Ok(game) => {
                let game_author = game.author.clone();
                if confirmation_required && self.config.response_time_limit.is_some() {
                    let response_time = self.config.response_time_limit.unwrap();
                    self.save_state.current_state = GameJamState::Waiting {
                        time_limit: response_time as f32,
                        game,
                    };
                    self.update_status(&format!("Waiting for response from {}", game_author));
                    Some(format!(
                        "@{}, we are about to play your game. Please reply in {} seconds.",
                        game_author, response_time
                    ))
                } else {
                    self.set_current(Some(game))
                }
            }
            Err(reply) => {
                self.set_current(None);
                Some(reply)
            }
        };
        self.save_games().unwrap();
        reply
    }

    pub fn skip(&mut self, auto_next: bool) -> Response {
        let state = std::mem::take(&mut self.save_state.current_state);
        match state {
            GameJamState::Playing { game } | GameJamState::Waiting { game, .. } => {
                self.save_state.current_state = GameJamState::Idle;
                self.save_state.skipped.push(game);
                let reply = "Game has been skipped.".to_owned();
                let reply = if auto_next {
                    self.next(None, true).unwrap_or(reply)
                } else {
                    self.save_games().unwrap();
                    reply
                };
                Some(reply)
            }
            state => {
                self.save_state.current_state = state;
                Some("Not playing any game at the moment.".to_owned())
            }
        }
    }

    fn skip_all(&mut self) -> Response {
        // self.skip(false);
        for game in self.save_state.returned_queue.drain(..) {
            self.save_state.skipped.push(game);
        }
        for game in self.save_state.games_queue.drain(..) {
            self.save_state.skipped.push(game);
        }
        self.save_games().unwrap();
        Some(format!(
            "All games from the queue are moved to the skipped list."
        ))
    }

    fn unskip(&mut self, author_name: Option<String>) -> Response {
        let mut reply = String::new();

        let state = std::mem::take(&mut self.save_state.current_state);
        match state {
            GameJamState::Playing { game } | GameJamState::Waiting { game, .. } => {
                self.save_state.returned_queue.push_front(game);
                reply.push_str("Current game has been put at the front of the queue. ");
            }
            _ => (),
        }

        match author_name {
            Some(author_name) => {
                let skipped = self
                    .save_state
                    .skipped
                    .iter()
                    .enumerate()
                    .find_map(|(index, game)| {
                        if game.author == author_name {
                            Some(index)
                        } else {
                            None
                        }
                    })
                    .map(|index| self.save_state.skipped.remove(index));
                match self.set_current(skipped) {
                    Some(set_reply) => reply.push_str(&set_reply),
                    None => reply.push_str(&format!("No game from {} found", author_name)),
                }
            }
            None => {
                if self.save_state.skipped.len() > 0 {
                    let skipped = self
                        .save_state
                        .skipped
                        .remove(self.save_state.skipped.len() - 1);
                    match self.set_current(Some(skipped)) {
                        Some(set_reply) => reply.push_str(&set_reply),
                        None => (),
                    }
                } else {
                    reply.push_str("No game has been skipped yet")
                }
            }
        }

        self.save_games().unwrap();
        Some(reply)
    }

    fn check_link(&self, game_link: &str) -> bool {
        if let Some(link_start) = &self.config.link_start {
            game_link.starts_with(link_start)
        } else {
            true
        }
    }

    fn submit(&mut self, game_link: String, sender: String) -> Response {
        // Check if submissions are closed
        if !self.save_state.is_open {
            return Some(
                "The queue is closed. You can not submit your game at the moment.".to_owned(),
            );
        }

        // Check if the link is legal
        if !self.check_link(&game_link) {
            return Some(format!("@{}, that link can not be submitted", sender));
        }

        // Check if the sender has already submitted a game
        let same_author = self.find_game(|game| game.author == sender);
        if !self.config.multiple_submissions && same_author.is_some() {
            return Some(format!(
                "@{}, you can not submit more than one game",
                sender
            ));
        }

        // Check if a game with the same link was already submitted
        let same_name = self.find_game(|game| game.link == game_link);
        if let Some((game, game_type)) = same_name {
            // Check if the author is different
            if game.author != sender {
                return Some(format!(
                    "@{}, that game has already been submitted by {}",
                    sender, game.author
                ));
            }

            let response = match game_type {
                GameType::Queued => {
                    format!("@{}, that game has already been submitted.", sender)
                }
                GameType::Current => {
                    format!("@{}, we are playing that game right now!", sender)
                }
                GameType::Skipped => format!(
                    "@{}, that game was skipped. You may return to the queue using !return command",
                    sender
                ),
                GameType::Played => format!("@{}, we have already played that game.", sender),
            };
            return Some(response);
        }

        self.save_state
            .games_queue
            .push_back(Game::new(sender.clone(), game_link));
        self.save_games().unwrap();
        return Some(format!("@{}, your game has been submitted!", sender));
    }

    fn raffle_start(&mut self) -> Response {
        match &self.save_state.current_state {
            GameJamState::Raffle { .. } => Some(format!(
                "The raffle is in progress. Type !join to join the raffle."
            )),
            _ => {
                self.save_state.current_state = GameJamState::Raffle {
                    joined: HashMap::new(),
                };
                self.set_current(None);
                self.update_status("The raffle is in progress. Type !join to join the raffle!");
                Some(format!(
                    "The raffle has started! Type !join to join the raffle."
                ))
            }
        }
    }

    fn raffle_finish(&mut self) -> Response {
        match &mut self.save_state.current_state {
            GameJamState::Raffle { joined } => {
                let joined = std::mem::take(joined);

                // Increase saved weights
                for viewer in joined.keys() {
                    *self
                        .save_state
                        .raffle_viewer_weights
                        .get_mut(viewer)
                        .unwrap() += 1;
                }

                // Select random
                let joined = joined.into_iter().collect::<Vec<(String, u32)>>();
                let chosen = joined.choose_weighted(&mut rand::thread_rng(), |&(_, weight)| weight);
                let reply = match chosen {
                    Ok((sender, _)) => {
                        match self.remove_game(sender) {
                            Some(game) => self.set_current(Some(game)),
                            None => {
                                // The winner has not submitted a game
                                self.save_state.raffle_viewer_weights.remove(sender);
                                Some(format!("{} has won the raffle!", sender))
                            }
                        }
                    }
                    Err(_) => {
                        self.save_state.current_state = GameJamState::Idle;
                        Some(format!("Noone entered the raffle :("))
                    }
                };

                self.save_games().unwrap();
                reply
            }
            _ => Some(format!("The raffle should be started first: !raffle")),
        }
    }

    fn raffle_join(&mut self, sender: String) -> Response {
        match &mut self.save_state.current_state {
            GameJamState::Raffle { joined } => {
                // If we have not played their game, then submit it
                if !self.played_games.iter().any(|game| game.author == sender) {
                    // Get weight
                    let weight = *self
                        .save_state
                        .raffle_viewer_weights
                        .entry(sender.clone())
                        .or_insert(self.config.raffle_default_weight);

                    // Join
                    joined.insert(sender, weight);
                }
            }
            _ => (),
        }
        None
    }

    fn raffle_cancel(&mut self) -> Response {
        match &mut self.save_state.current_state {
            GameJamState::Raffle { .. } => {
                self.save_state.current_state = GameJamState::Idle;
                self.save_games().unwrap();
                Some(format!("Raffle is now inactive"))
            }
            _ => Some(format!(
                "Raffle is not active anyway. Start the raffle with !raffle"
            )),
        }
    }

    pub fn return_game(&mut self, author_name: &str) -> Response {
        if !self.save_state.is_open {
            return None;
        }

        let reply = if let Some(index) = self
            .save_state
            .skipped
            .iter()
            .enumerate()
            .find(|(_, game)| game.author == author_name)
            .map(|(index, _)| index)
        {
            let game = self.save_state.skipped.remove(index);
            match self.config.return_mode {
                ReturnMode::Front => self.save_state.returned_queue.push_back(game),
                ReturnMode::Back => self.save_state.games_queue.push_back(game),
            }
            self.save_games().unwrap();
            Some(format!(
                "@{}, your game was returned to the queue",
                author_name
            ))
        } else {
            None
        };
        reply
    }

    fn luck(&self, author_name: &str) -> Response {
        // Get luck
        let luck = self.save_state.raffle_viewer_weights.get(author_name);

        // Choose reply
        let reply = luck
            .map(|luck| format!("@{}, your current luck level is {}", author_name, luck))
            .unwrap_or(format!(
                "@{}, you have regular luck level {}",
                author_name, self.config.raffle_default_weight
            ));

        Some(reply)
    }

    fn force(&mut self) -> Response {
        let state = std::mem::take(&mut self.save_state.current_state);
        match state {
            GameJamState::Waiting { game, .. } => self.set_current(Some(game)),
            state => {
                self.save_state.current_state = state;
                Some("Not waiting for response at the moment".to_owned())
            }
        }
    }

    fn queue(&self, sender_name: &str) -> Response {
        let mut reply = String::new();
        if self.config.queue_mode {
            if let Some((pos, _)) = self
                .save_state
                .queue()
                .enumerate()
                .find(|(_, game)| game.author == *sender_name)
            {
                reply.push_str(&format!(
                    "@{}, your game is {} in the queue. ",
                    sender_name,
                    pos + 1
                ));
            }
        }

        if self
            .save_state
            .skipped
            .iter()
            .any(|game| game.author == *sender_name)
        {
            reply.push_str(&format!(
                "@{}, your game was skipped. You may return to the queue using !return command. ",
                sender_name
            ))
        }

        if let Some(config) = &self.config.google_sheet_config {
            reply.push_str(&format!("Look at the current queue at: https://docs.google.com/spreadsheets/d/{}/edit#gid=0", config.sheet_id))
        } else if self.config.queue_mode {
            let mut reply = String::new();
            let games_count = self.save_state.queue().count();
            if games_count == 0 {
                reply.push_str("The queue is empty");
            } else {
                reply.push_str(&format!("There are {} games in the queue", games_count));
            }
        }

        if !reply.is_empty() {
            Some(reply)
        } else {
            None
        }
    }

    pub fn commands() -> Commands<Self, Sender> {
        Commands {
            commands: vec![
                CommandNode::Argument {
                    argument_type: ArgumentType::Word,
                    child_nodes: vec![CommandNode::Final {
                        authority_level: AuthorityLevel::Viewer as usize,
                        command: Arc::new(|bot, sender, mut args| {
                            let game_link = args.remove(0);
                            if bot.config.allow_direct_link_submit
                                && bot.config.link_start.is_some()
                                && bot.check_link(&game_link)
                            {
                                return bot.submit(game_link, sender.name.clone());
                            }
                            None
                        }),
                    }],
                },
                CommandNode::Literal {
                    literals: vec!["!submit".to_owned()],
                    child_nodes: vec![CommandNode::Argument {
                        argument_type: ArgumentType::Word,
                        child_nodes: vec![CommandNode::Final {
                            authority_level: AuthorityLevel::Viewer as usize,
                            command: Arc::new(|bot, sender, mut args| {
                                let game_link = args.remove(0);
                                bot.submit(game_link, sender.name.clone())
                            }),
                        }],
                    }],
                },
                CommandNode::Literal {
                    literals: vec!["!return".to_owned()],
                    child_nodes: vec![CommandNode::Final {
                        authority_level: AuthorityLevel::Viewer as usize,
                        command: Arc::new(|bot, sender, _| bot.return_game(&sender.name)),
                    }],
                },
                CommandNode::Literal {
                    literals: vec!["!next".to_owned()],
                    child_nodes: vec![
                        CommandNode::Final {
                            authority_level: AuthorityLevel::Broadcaster as usize,
                            command: Arc::new(|bot, _, _| bot.next(None, true)),
                        },
                        CommandNode::Argument {
                            argument_type: ArgumentType::Word,
                            child_nodes: vec![CommandNode::Final {
                                authority_level: AuthorityLevel::Broadcaster as usize,
                                command: Arc::new(|bot, _, mut args| {
                                    let author_name = args.remove(0);
                                    bot.next(Some(author_name), false)
                                }),
                            }],
                        },
                    ],
                },
                CommandNode::Literal {
                    literals: vec!["!cancel".to_owned()],
                    child_nodes: vec![
                        CommandNode::Final {
                            authority_level: AuthorityLevel::Viewer as usize,
                            command: Arc::new(|bot, sender, _| {
                                bot.remove_game_response(&sender.name)
                            }),
                        },
                        CommandNode::Argument {
                            argument_type: ArgumentType::Word,
                            child_nodes: vec![CommandNode::Final {
                                authority_level: AuthorityLevel::Moderator as usize,
                                command: Arc::new(|bot, _, mut args| {
                                    let author_name = args.remove(0);
                                    bot.remove_game_response(&author_name)
                                }),
                            }],
                        },
                    ],
                },
                CommandNode::Literal {
                    literals: vec!["!queue".to_owned(), "!list".to_owned()],
                    child_nodes: vec![CommandNode::Final {
                        authority_level: AuthorityLevel::Viewer as usize,
                        command: Arc::new(|bot, sender, _| bot.queue(&sender.name)),
                    }],
                },
                CommandNode::Literal {
                    literals: vec!["!current".to_owned()],
                    child_nodes: vec![CommandNode::Final {
                        authority_level: AuthorityLevel::Viewer as usize,
                        command: Arc::new(|bot, _, _| match &bot.save_state.current_state {
                            GameJamState::Playing { game } => {
                                Some(format!("Current game is: {}", game.to_string_link(false)))
                            }
                            _ => Some("Not playing any game at the moment".to_owned()),
                        }),
                    }],
                },
                CommandNode::Literal {
                    literals: vec!["!skip".to_owned()],
                    child_nodes: vec![
                        CommandNode::Literal {
                            literals: vec!["next".to_owned()],
                            child_nodes: vec![CommandNode::Final {
                                authority_level: AuthorityLevel::Broadcaster as usize,
                                command: Arc::new(|bot, _, _| bot.skip(true)),
                            }],
                        },
                        CommandNode::Literal {
                            literals: vec!["all".to_owned()],
                            child_nodes: vec![CommandNode::Final {
                                authority_level: AuthorityLevel::Broadcaster as usize,
                                command: Arc::new(|bot, _, _| bot.skip_all()),
                            }],
                        },
                        CommandNode::Final {
                            authority_level: AuthorityLevel::Broadcaster as usize,
                            command: Arc::new(|bot, _, _| bot.skip(false)),
                        },
                    ],
                },
                CommandNode::Literal {
                    literals: vec!["!unskip".to_owned()],
                    child_nodes: vec![
                        CommandNode::Final {
                            authority_level: AuthorityLevel::Broadcaster as usize,
                            command: Arc::new(|bot, _, _| bot.unskip(None)),
                        },
                        CommandNode::Argument {
                            argument_type: ArgumentType::Word,
                            child_nodes: vec![CommandNode::Final {
                                authority_level: AuthorityLevel::Broadcaster as usize,
                                command: Arc::new(|bot, _, mut args| {
                                    let author_name = args.remove(0);
                                    bot.unskip(Some(author_name))
                                }),
                            }],
                        },
                    ],
                },
                CommandNode::Literal {
                    literals: vec!["!stop".to_owned()],
                    child_nodes: vec![CommandNode::Final {
                        authority_level: AuthorityLevel::Broadcaster as usize,
                        command: Arc::new(|bot, _, _| {
                            bot.set_current(None);
                            Some("Current game set to None".to_owned())
                        }),
                    }],
                },
                CommandNode::Literal {
                    literals: vec!["!force".to_owned()],
                    child_nodes: vec![CommandNode::Final {
                        authority_level: AuthorityLevel::Moderator as usize,
                        command: Arc::new(|bot, _, _| bot.force()),
                    }],
                },
                CommandNode::Literal {
                    literals: vec!["!close".to_owned()],
                    child_nodes: vec![CommandNode::Final {
                        authority_level: AuthorityLevel::Moderator as usize,
                        command: Arc::new(|bot, _, _| {
                            bot.save_state.is_open = false;
                            bot.save_games().unwrap();
                            Some("The queue is now closed".to_owned())
                        }),
                    }],
                },
                CommandNode::Literal {
                    literals: vec!["!open".to_owned()],
                    child_nodes: vec![CommandNode::Final {
                        authority_level: AuthorityLevel::Moderator as usize,
                        command: Arc::new(|bot, _, _| {
                            bot.save_state.is_open = true;
                            bot.save_games().unwrap();
                            Some("The queue is now open".to_owned())
                        }),
                    }],
                },
                CommandNode::Literal {
                    literals: vec!["!raffle".to_owned()],
                    child_nodes: vec![
                        CommandNode::Final {
                            authority_level: AuthorityLevel::Broadcaster as usize,
                            command: Arc::new(|bot, _, _| bot.raffle_start()),
                        },
                        CommandNode::Literal {
                            literals: vec!["finish".to_owned()],
                            child_nodes: vec![CommandNode::Final {
                                authority_level: AuthorityLevel::Broadcaster as usize,
                                command: Arc::new(|bot, _, _| bot.raffle_finish()),
                            }],
                        },
                        CommandNode::Literal {
                            literals: vec!["cancel".to_owned()],
                            child_nodes: vec![CommandNode::Final {
                                authority_level: AuthorityLevel::Broadcaster as usize,
                                command: Arc::new(|bot, _, _| bot.raffle_cancel()),
                            }],
                        },
                    ],
                },
                CommandNode::Literal {
                    literals: vec!["!join".to_owned()],
                    child_nodes: vec![CommandNode::Final {
                        authority_level: AuthorityLevel::Viewer as usize,
                        command: Arc::new(|bot, sender, _| bot.raffle_join(sender.name.clone())),
                    }],
                },
                CommandNode::Literal {
                    literals: vec!["!luck".to_owned()],
                    child_nodes: vec![CommandNode::Final {
                        authority_level: AuthorityLevel::Viewer as usize,
                        command: Arc::new(|bot, sender, _| bot.luck(&sender.name)),
                    }],
                },
            ],
        }
    }
}
