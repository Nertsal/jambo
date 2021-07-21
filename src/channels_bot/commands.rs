use std::sync::Arc;

use super::*;

impl CommandBot<Self> for ChannelsBot {
    fn get_commands(&self) -> &BotCommands<Self> {
        &self.commands
    }

    fn get_cli(&self) -> &CLI {
        &self.cli
    }
}

macro_rules! bots_map {
    ( $( $b:ident ),* ) => {
        {
            let mut bots: HashMap<String, Box<fn(&CLI, &str) -> Box<dyn Bot>>> = HashMap::new();
            $(
                bots.insert($b::name().to_owned(), Box::new($b::new));
            )*
            bots
        }
    };
}

impl ChannelsBot {
    pub fn available_bots() -> HashMap<String, Box<fn(&CLI, &str) -> Box<dyn Bot>>> {
        bots_map!(CustomBot, GameJamBot, QuoteBot, TimerBot, VoteBot)
    }

    pub fn spawn_bot(&mut self, bot_name: &str) -> Response {
        let (response, new_bot) = if self.active_bots.contains_key(bot_name) {
            (Some(format!("{} is already active", bot_name)), None)
        } else {
            match self.new_bot(bot_name) {
                Some(new_bot) => (Some(format!("{} is now active", bot_name)), Some(new_bot)),
                None => (None, None),
            }
        };
        if let Some(new_bot) = new_bot {
            self.log(LogType::Info, &format!("Spawned bot {}", bot_name));
            self.active_bots.insert(bot_name.to_owned(), new_bot);
        }
        self.save_bots().unwrap();
        response
    }

    fn disable_bot(&mut self, bot_name: &str) -> Response {
        let bot = self.active_bots.remove(bot_name);
        let response = bot.map(|bot| format!("{} is no longer active", bot.name()));
        self.save_bots().unwrap();
        response
    }

    fn reset_bot(&mut self, bot_name: &str) -> Response {
        self.disable_bot(bot_name);
        self.spawn_bot(bot_name)
            .map(|_| format!("{} is reset", bot_name))
    }

    fn save_bots(&self) -> std::io::Result<()> {
        let active_bots = self.active_bots().unwrap();
        let file = std::io::BufWriter::new(std::fs::File::create("config/active_bots.json")?);
        serde_json::to_writer(file, &active_bots)?;
        Ok(())
    }

    fn active_bots(&self) -> Result<ActiveBots, ()> {
        let mut active_bots = HashSet::with_capacity(self.active_bots.len());
        for bot_name in self.active_bots.keys() {
            active_bots.insert(bot_name.to_owned());
        }
        Ok(active_bots)
    }

    fn new_bot(&self, bot_name: &str) -> Option<Box<dyn Bot>> {
        self.available_bots
            .get(bot_name)
            .map(|f| f(&self.cli, &self.channel_login))
    }

    pub fn commands<'a>(available_bots: impl Iterator<Item = &'a String>) -> BotCommands<Self> {
        BotCommands {
            commands: vec![
                CommandNode::Literal {
                    literals: vec!["!shutdown".to_owned()],
                    child_nodes: vec![CommandNode::Final {
                        authority_level: AuthorityLevel::Broadcaster,
                        command: Arc::new(|bot, _, _| {
                            bot.queue_shutdown = true;
                            bot.log(LogType::Info, "Shutting down...");
                            Some(format!("Shutting down..."))
                        }),
                    }],
                },
                CommandNode::ArgumentChoice {
                    choices: vec![
                        "!enable".to_owned(),
                        "!disable".to_owned(),
                        "!reset".to_owned(),
                    ],
                    child_nodes: vec![CommandNode::ArgumentChoice {
                        choices: available_bots.map(|name| name.clone()).collect(),
                        child_nodes: vec![CommandNode::Final {
                            authority_level: AuthorityLevel::Moderator,
                            command: Arc::new(|bot, _, args| {
                                let bot_name = args[1].as_str();
                                let response = match args[0].as_str() {
                                    "!enable" => bot.spawn_bot(bot_name),
                                    "!disable" => bot.disable_bot(bot_name),
                                    "!reset" => bot.reset_bot(bot_name),
                                    _ => unreachable!(),
                                };
                                let completer = Arc::new(CommandCompleter {
                                    completion_tree: bot.get_completion_tree(),
                                });
                                bot.get_cli().set_completer(completer);
                                response
                            }),
                        }],
                    }],
                },
            ],
        }
    }
}
