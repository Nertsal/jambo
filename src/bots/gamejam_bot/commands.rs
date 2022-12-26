use rand::seq::SliceRandom;
use std::sync::Arc;

use super::*;

impl GamejamBot {
    pub fn set_current(&mut self, game: Option<Submission>) -> Response {
        let state = std::mem::take(&mut self.state.current_state);
        match state {
            GameJamState::Playing { game } | GameJamState::Waiting { game, .. } => {
                self.state.submissions.played_games.push(game);
                save_into(&self.state.submissions.played_games, PLAYED_GAMES_FILE).unwrap();
            }
            _ => (),
        }

        let reply = match game {
            Some(game) => {
                self.update_status(&format!("Playing {}", game.to_string_name(true)));
                let reply = format!("Now playing {}. ", game.to_string_link(true));
                self.state.raffle_weights.remove(&game.link);
                self.state.current_state = GameJamState::Playing { game };
                Some(reply)
            }
            None => {
                self.update_status("Not playing a game");
                self.state.current_state = GameJamState::Idle;
                None
            }
        };

        self.save_games().unwrap();
        reply
    }

    fn find_game(
        &self,
        predicate: impl Fn(&Submission) -> bool,
    ) -> Option<(&Submission, GameType)> {
        // Check current
        if let Some(game) = self.state.current_state.current() {
            if predicate(game) {
                return Some((game, GameType::Current));
            }
        }

        // Check submissions
        self.state.submissions.find_game(predicate)
    }

    fn find_game_mut(
        &mut self,
        predicate: impl Fn(&Submission) -> bool,
    ) -> Option<(&mut Submission, GameType)> {
        // Check current
        if let Some(game) = self.state.current_state.current_mut() {
            if predicate(game) {
                return Some((game, GameType::Current));
            }
        }

        // Check submissions
        self.state.submissions.find_game_mut(predicate)
    }

    fn remove_game_response(&mut self, author_name: &String, check_main_author: bool) -> Response {
        match self
            .state
            .submissions
            .remove_game(|game| !check_main_author || game.authors[0] == *author_name)
        {
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

    pub fn next(&mut self, author_name: Option<&String>, confirmation_required: bool) -> Response {
        let game = match author_name {
            Some(author_name) => match self
                .state
                .submissions
                .remove_game(|game| game.authors.contains(author_name))
            {
                Some(game) => Ok(game),
                None => Err(format!("Couldn't find a game from {}", author_name)),
            },
            None => match self.state.submissions.queue.next() {
                Some(game) => Ok(game),
                None => Err("The queue is empty. !submit <your game>. ".to_string()),
            },
        };

        let reply = match game {
            Ok(game) => {
                let game_author = game.authors[0].clone();
                if confirmation_required && self.config.response_time_limit.is_some() {
                    let response_time = self.config.response_time_limit.unwrap();
                    self.state.current_state = GameJamState::Waiting {
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
        let state = std::mem::take(&mut self.state.current_state);
        match state {
            GameJamState::Playing { game } | GameJamState::Waiting { game, .. } => {
                self.state.current_state = GameJamState::Idle;
                self.state.submissions.skipped.push(game);
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
                self.state.current_state = state;
                Some("Not playing any game at the moment.".to_owned())
            }
        }
    }

    fn skip_all(&mut self) -> Response {
        for game in self.state.submissions.queue.drain_all() {
            self.state.submissions.skipped.push(game);
        }
        self.save_games().unwrap();
        Some("All games from the queue are moved to the skipped list.".to_string())
    }

    fn unskip(&mut self, author_name: Option<&String>) -> Response {
        let mut reply = String::new();

        let state = std::mem::take(&mut self.state.current_state);
        match state {
            GameJamState::Playing { game } | GameJamState::Waiting { game, .. } => {
                self.state.submissions.queue.return_game_front(game);
                reply.push_str("Current game has been put at the front of the queue. ");
            }
            _ => (),
        }

        match author_name {
            Some(author_name) => {
                let skipped = self
                    .state
                    .submissions
                    .skipped
                    .iter()
                    .enumerate()
                    .find_map(|(index, game)| {
                        if game.authors.contains(author_name) {
                            Some(index)
                        } else {
                            None
                        }
                    })
                    .map(|index| self.state.submissions.skipped.remove(index));
                match self.set_current(skipped) {
                    Some(set_reply) => reply.push_str(&set_reply),
                    None => reply.push_str(&format!("No game from {} found", author_name)),
                }
            }
            None => {
                if !self.state.submissions.skipped.is_empty() {
                    let skipped = self
                        .state
                        .submissions
                        .skipped
                        .remove(self.state.submissions.skipped.len() - 1);
                    if let Some(set_reply) = self.set_current(Some(skipped)) {
                        reply.push_str(&set_reply)
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
        if !self.state.is_queue_open {
            return Some(
                "The queue is closed. You can not submit your game at the moment.".to_owned(),
            );
        }

        // Check if the link is legal
        if !self.check_link(&game_link) {
            return Some(format!("@{}, that link can not be submitted", sender));
        }

        // Check if the sender has already submitted a game
        let same_author = self.find_game(|game| game.authors.contains(&sender));
        if !self.config.multiple_submissions && same_author.is_some() {
            return Some(format!(
                "@{}, you can not submit more than one game",
                sender
            ));
        }

        // Check if a game with the same link was already submitted
        let allow_multiple_authors_submits = self.config.allow_multiple_authors_submit;
        let same_name = self.find_game_mut(|game| game.link == game_link);
        if let Some((game, game_type)) = same_name {
            // Check if the game has already been played
            if let GameType::Played = &game_type {
                return Some(format!("@{}, we have already played that game.", sender));
            }

            // Check if sender should be added as another author
            if allow_multiple_authors_submits && !game.authors.contains(&sender) {
                let response = format!(
                    "@{}, you have been marked as another author of this game",
                    sender
                );
                game.authors.push(sender);
                self.save_games().unwrap();
                return Some(response);
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
                _ => unreachable!(),
            };
            return Some(response);
        }

        let response = format!("@{}, your game has been submitted!", sender);

        self.state
            .submissions
            .queue
            .queue_game(Submission::new(vec![sender], game_link));
        self.save_games().unwrap();

        Some(response)
    }

    fn edit_game(
        &mut self,
        sender: &String,
        check_main_author: bool,
        predicate: impl Fn(&Submission) -> bool,
    ) -> Result<&mut Submission, Response> {
        let game = self.find_game_mut(predicate);
        if let Some((game, game_type)) = game {
            // Check main author
            if check_main_author && game.authors[0] != *sender {
                let response = format!("@{}, you do not have enough rights", sender);
                return Err(Some(response));
            }

            match game_type {
                GameType::Current | GameType::Queued | GameType::Skipped => return Ok(game),
                GameType::Played => {
                    let response = format!("@{}, you cannot edit played games", sender);
                    return Err(Some(response));
                }
            }
        }

        // No game from sender
        let response = format!("@{}, you have not submitted a game yet", sender);
        Err(Some(response))
    }

    fn authors_add(
        &mut self,
        sender: &String,
        other_author: String,
        check_main_author: bool,
        predicate: impl Fn(&Submission) -> bool,
    ) -> Response {
        let game = self.edit_game(sender, check_main_author, predicate);
        match game {
            Err(response) => response,
            Ok(game) => {
                let response = format!(
                    "@{}, {} was marked as another author of the game",
                    sender, other_author
                );
                game.authors.push(other_author);
                self.save_games().unwrap();
                Some(response)
            }
        }
    }

    fn authors_remove(
        &mut self,
        sender: &String,
        other_author: &String,
        check_main_author: bool,
        predicate: impl Fn(&Submission) -> bool,
    ) -> Response {
        let game = self.edit_game(sender, check_main_author, predicate);
        match game {
            Err(response) => response,
            Ok(game) => {
                // Removing every author is prohibited
                if game.authors.len() == 1 {
                    let response = format!(
                        "@{}, you cannot remove the only author of the game. Call !cancel instead",
                        sender
                    );
                    return Some(response);
                }

                let index = game
                    .authors
                    .iter()
                    .enumerate()
                    .find(|(_, author)| **author == *other_author)
                    .map(|(index, _)| index);
                match index {
                    Some(index) => {
                        game.authors.remove(index);
                        self.save_games().unwrap();
                        let response = format!(
                            "@{}, {} was removed from the author list of the game",
                            sender, other_author
                        );
                        Some(response)
                    }
                    None => {
                        let response = format!("@{}, {} was not found", sender, other_author);
                        Some(response)
                    }
                }
            }
        }
    }

    fn raffle_start(&mut self) -> Response {
        match &self.state.current_state {
            GameJamState::Raffle { .. } => {
                Some("The raffle is in progress. Type !join to join the raffle.".to_string())
            }
            _ => {
                self.set_current(None);
                self.state.current_state = GameJamState::Raffle {
                    joined: HashMap::new(),
                };
                self.update_status("The raffle is in progress. Type !join to join the raffle!");
                Some("The raffle has started! Type !join to join the raffle.".to_string())
            }
        }
    }

    fn raffle_finish(&mut self) -> Response {
        match &mut self.state.current_state {
            GameJamState::Raffle { joined } => {
                let joined = std::mem::take(joined);

                // Increase saved weights
                for game_link in joined.keys() {
                    *self.state.raffle_weights.get_mut(game_link).unwrap() += 1;
                }

                // Select random
                let joined = joined.into_iter().collect::<Vec<(String, u32)>>();
                let chosen = joined.choose_weighted(&mut rand::thread_rng(), |&(_, weight)| weight);
                let reply = match chosen {
                    Ok((game_link, _)) => {
                        match self
                            .state
                            .submissions
                            .remove_game(|game| game.link == *game_link)
                        {
                            Some(game) => self.set_current(Some(game)),
                            None => {
                                unreachable!()
                                // The winner has not submitted a game
                                // self.state.raffle_weights.remove(sender);
                                // Some(format!("{} has won the raffle!", sender))
                            }
                        }
                    }
                    Err(_) => {
                        self.state.current_state = GameJamState::Idle;
                        Some("Noone entered the raffle :(".to_string())
                    }
                };

                self.save_games().unwrap();
                reply
            }
            _ => Some("The raffle should be started first: !raffle".to_string()),
        }
    }

    fn raffle_join(&mut self, sender: String) -> Response {
        if let GameJamState::Raffle { joined } = &mut self.state.current_state {
            // Find the game from sender
            // Only those who have submitted a game and whose game has not been played yet
            // are allowed to join the raffle
            let game = self
                .state
                .submissions
                .find_game(|game| game.authors.contains(&sender));
            match game {
                Some((game, game_type)) => match game_type {
                    GameType::Played => {
                        // The game has already been played
                        let response = format!("@{}, we have already played your game", sender);
                        Some(response)
                    }
                    _ => {
                        let game_link = game.link.clone();
                        // Get weight
                        let weight = *self
                            .state
                            .raffle_weights
                            .entry(game_link.clone())
                            .or_insert(self.config.raffle_default_weight);

                        // Join
                        joined.insert(game_link, weight);

                        // Return with no response
                        None
                    }
                },
                None => {
                    // Did not find a game from sender
                    let response = format!("@{}, you cannot join the raffle", sender);
                    Some(response)
                }
            }
        } else {
            // Not doing a raffle at the moment
            None
        }
    }

    fn raffle_cancel(&mut self) -> Response {
        match &mut self.state.current_state {
            GameJamState::Raffle { .. } => {
                self.state.current_state = GameJamState::Idle;
                self.save_games().unwrap();
                Some("Raffle is now inactive".to_string())
            }
            _ => Some("Raffle is not active anyway. Start the raffle with !raffle".to_string()),
        }
    }

    pub fn return_game(&mut self, author_name: &String) -> Response {
        if !self.state.is_queue_open {
            return None;
        }

        let reply = if let Some(index) = self
            .state
            .submissions
            .skipped
            .iter()
            .enumerate()
            .find(|(_, game)| game.authors.contains(author_name))
            .map(|(index, _)| index)
        {
            let game = self.state.submissions.skipped.remove(index);
            match self.config.return_mode {
                ReturnMode::Front => self.state.submissions.queue.return_game(game),
                ReturnMode::Back => self.state.submissions.queue.queue_game(game),
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

    fn luck(&self, author_name: &String) -> Response {
        // Check for registered luck level
        let luck = self.state.raffle_weights.get(author_name).copied();

        let luck = match luck {
            Some(luck) => luck,
            None => {
                // Check if the viewer can join the raffle, hence their luck level is default
                let game = self.find_game(|game| game.authors.contains(author_name));

                match game {
                    None => {
                        let response =
                            format!("@{}, you need to first submit your game!", author_name);
                        return Some(response);
                    }
                    Some((_, game_type)) => match game_type {
                        GameType::Queued | GameType::Skipped => self.config.raffle_default_weight,
                        _ => {
                            let response = format!(
                                "@{}, you can no longer participate in raffles!",
                                author_name
                            );
                            return Some(response);
                        }
                    },
                }
            }
        };

        // Respond
        let response = format!("@{}, your current luck level is {}", author_name, luck);
        Some(response)
    }

    fn force(&mut self) -> Response {
        let state = std::mem::take(&mut self.state.current_state);
        match state {
            GameJamState::Waiting { game, .. } => self.set_current(Some(game)),
            state => {
                self.state.current_state = state;
                Some("Not waiting for response at the moment".to_owned())
            }
        }
    }

    fn queue(&self, sender_name: &String) -> Response {
        let mut reply = String::new();
        if self.config.queue_mode {
            if let Some((pos, _)) = self
                .state
                .submissions
                .queue
                .get_queue()
                .enumerate()
                .find(|(_, game)| game.authors.contains(sender_name))
            {
                reply.push_str(&format!(
                    "@{}, your game is {} in the queue. ",
                    sender_name,
                    pos + 1
                ));
            }
        }

        if self
            .state
            .submissions
            .skipped
            .iter()
            .any(|game| game.authors.contains(sender_name))
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
            let games_count = self.state.submissions.queue.get_queue().count();
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

    pub fn commands() -> Commands<Self> {
        let direct_submit = CommandBuilder::<Self>::new().word().finalize(
            true,
            AuthorityLevel::Viewer as usize,
            Arc::new(|bot, sender, mut args| {
                let game_link = args.remove(0);
                if bot.config.allow_direct_link_submit
                    && bot.config.link_start.is_some()
                    && bot.check_link(&game_link)
                {
                    return bot.submit(game_link, sender.name.clone());
                }
                None
            }),
        );

        let submit = CommandBuilder::<Self>::new()
            .literal(["!submit"])
            .word()
            .finalize(
                true,
                AuthorityLevel::Viewer as usize,
                Arc::new(|bot, sender, mut args| {
                    let game_link = args.remove(0);
                    bot.submit(game_link, sender.name.clone())
                }),
            );

        let retur = CommandBuilder::<Self>::new().literal(["!return"]).finalize(
            true,
            AuthorityLevel::Viewer as usize,
            Arc::new(|bot, sender, _| bot.return_game(&sender.name)),
        );

        let next_queue = CommandBuilder::<Self>::new().finalize(
            true,
            AuthorityLevel::Broadcaster as usize,
            Arc::new(|bot, _, _| bot.next(None, true)),
        );

        let next_author = CommandBuilder::<Self>::new().word().finalize(
            true,
            AuthorityLevel::Broadcaster as usize,
            Arc::new(|bot, _, mut args| {
                let author_name = args.remove(0);
                bot.next(Some(&author_name), false)
            }),
        );

        let next = CommandBuilder::new()
            .literal(["!next"])
            .split([next_queue, next_author]);

        let cancel_sender = CommandBuilder::<Self>::new().finalize(
            true,
            AuthorityLevel::Viewer as usize,
            Arc::new(|bot, sender, _| bot.remove_game_response(&sender.name, true)),
        );

        let cancel_author = CommandBuilder::<Self>::new().word().finalize(
            true,
            AuthorityLevel::Moderator as usize,
            Arc::new(|bot, _, mut args| {
                let author_name = args.remove(0);
                bot.remove_game_response(&author_name, false)
            }),
        );

        let cancel = CommandBuilder::new()
            .literal(["!cancel"])
            .split([cancel_sender, cancel_author]);

        let list = CommandBuilder::<Self>::new()
            .literal(["!queue", "!list"])
            .finalize(
                true,
                AuthorityLevel::Viewer as usize,
                Arc::new(|bot, sender, _| bot.queue(&sender.name)),
            );

        let current = CommandBuilder::<Self>::new()
            .literal(["!current"])
            .finalize(
                true,
                AuthorityLevel::Viewer as usize,
                Arc::new(|bot, _, _| match &bot.state.current_state {
                    GameJamState::Playing { game } => {
                        Some(format!("Current game is: {}", game.to_string_link(false)))
                    }
                    _ => Some("Not playing any game at the moment".to_owned()),
                }),
            );

        let skip_next = CommandBuilder::<Self>::new().literal(["next"]).finalize(
            true,
            AuthorityLevel::Broadcaster as usize,
            Arc::new(|bot, _, _| bot.skip(true)),
        );

        let skip_all = CommandBuilder::<Self>::new().literal(["all"]).finalize(
            true,
            AuthorityLevel::Broadcaster as usize,
            Arc::new(|bot, _, _| bot.skip_all()),
        );

        let skip_current = CommandBuilder::<Self>::new().finalize(
            true,
            AuthorityLevel::Broadcaster as usize,
            Arc::new(|bot, _, _| bot.skip(false)),
        );

        let skip =
            CommandBuilder::new()
                .literal(["!skip"])
                .split([skip_next, skip_all, skip_current]);

        let unskip_last = CommandBuilder::<Self>::new().finalize(
            true,
            AuthorityLevel::Broadcaster as usize,
            Arc::new(|bot, _, _| bot.unskip(None)),
        );

        let unskip_author = CommandBuilder::<Self>::new().word().finalize(
            true,
            AuthorityLevel::Broadcaster as usize,
            Arc::new(|bot, _, mut args| {
                let author_name = args.remove(0);
                bot.unskip(Some(&author_name))
            }),
        );

        let unskip = CommandBuilder::new()
            .literal(["!unskip"])
            .split([unskip_last, unskip_author]);

        let stop = CommandBuilder::<Self>::new().literal(["!stop"]).finalize(
            true,
            AuthorityLevel::Broadcaster as usize,
            Arc::new(|bot, _, _| {
                bot.set_current(None);
                Some("Current game set to None".to_owned())
            }),
        );

        let force = CommandBuilder::<Self>::new().literal(["!force"]).finalize(
            true,
            AuthorityLevel::Moderator as usize,
            Arc::new(|bot, _, _| bot.force()),
        );

        let close = CommandBuilder::<Self>::new().literal(["!close"]).finalize(
            true,
            AuthorityLevel::Moderator as usize,
            Arc::new(|bot, _, _| {
                bot.state.is_queue_open = false;
                bot.save_games().unwrap();
                Some("The queue is now closed".to_owned())
            }),
        );

        let open = CommandBuilder::<Self>::new().literal(["!open"]).finalize(
            true,
            AuthorityLevel::Moderator as usize,
            Arc::new(|bot, _, _| {
                bot.state.is_queue_open = true;
                bot.save_games().unwrap();
                Some("The queue is now open".to_owned())
            }),
        );

        let raffle_start = CommandBuilder::<Self>::new().finalize(
            true,
            AuthorityLevel::Broadcaster as usize,
            Arc::new(|bot, _, _| bot.raffle_start()),
        );

        let raffle_finish = CommandBuilder::<Self>::new().literal(["finish"]).finalize(
            true,
            AuthorityLevel::Broadcaster as usize,
            Arc::new(|bot, _, _| bot.raffle_finish()),
        );

        let raffle_cancel = CommandBuilder::<Self>::new().literal(["cancel"]).finalize(
            true,
            AuthorityLevel::Broadcaster as usize,
            Arc::new(|bot, _, _| bot.raffle_cancel()),
        );

        let raffle = CommandBuilder::new().literal(["!raffle"]).split([
            raffle_start,
            raffle_finish,
            raffle_cancel,
        ]);

        let join = CommandBuilder::<Self>::new().literal(["!join"]).finalize(
            true,
            AuthorityLevel::Viewer as usize,
            Arc::new(|bot, sender, _| bot.raffle_join(sender.name.clone())),
        );

        let luck = CommandBuilder::<Self>::new().literal(["!luck"]).finalize(
            true,
            AuthorityLevel::Viewer as usize,
            Arc::new(|bot, sender, _| bot.luck(&sender.name)),
        );

        let authors_add = CommandBuilder::new().literal(["add"]).word().split([
            CommandBuilder::<Self>::new().word().finalize(
                true,
                AuthorityLevel::Moderator as usize,
                Arc::new(|bot, sender, mut args| {
                    let game_link = args.remove(0);
                    let other_author = args.remove(0);
                    bot.authors_add(&sender.name, other_author, false, |game| {
                        game.link == game_link
                    })
                }),
            ),
            CommandBuilder::new().finalize(
                true,
                AuthorityLevel::Viewer as usize,
                Arc::new(|bot, sender, mut args| {
                    let other_author = args.remove(0);
                    bot.authors_add(&sender.name, other_author, true, |game| {
                        game.authors.contains(&sender.name)
                    })
                }),
            ),
        ]);

        let authors_remove = CommandBuilder::new().literal(["remove"]).word().split([
            CommandBuilder::<Self>::new().word().finalize(
                true,
                AuthorityLevel::Moderator as usize,
                Arc::new(|bot, sender, mut args| {
                    let game_link = args.remove(0);
                    let other_author = args.remove(0);
                    bot.authors_remove(&sender.name, &other_author, false, |game| {
                        game.link == game_link
                    })
                }),
            ),
            CommandBuilder::new().finalize(
                true,
                AuthorityLevel::Viewer as usize,
                Arc::new(|bot, sender, mut args| {
                    let other_author = args.remove(0);
                    bot.authors_remove(&sender.name, &other_author, true, |game| {
                        game.authors.contains(&sender.name)
                    })
                }),
            ),
        ]);

        let authors = CommandBuilder::new()
            .literal(["!authors"])
            .split([authors_add, authors_remove]);

        Commands {
            commands: vec![
                direct_submit,
                submit,
                retur,
                next,
                cancel,
                list,
                current,
                skip,
                unskip,
                stop,
                force,
                close,
                open,
                raffle,
                join,
                luck,
                authors,
            ],
        }
    }
}
