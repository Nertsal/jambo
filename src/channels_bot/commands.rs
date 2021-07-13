use std::sync::Arc;

use super::*;

impl CommandBot<Self> for ChannelsBot {
    fn get_commands(&self) -> &BotCommands<Self> {
        &self.commands
    }
}

macro_rules! bots_map {
    ( $( $b:ident ),* ) => {
        {
            let mut bots: HashMap<String, Box<fn(&str) -> Box<dyn Bot>>> = HashMap::new();
            $(
                bots.insert($b::name().to_owned(), Box::new($b::new));
            )*
            bots
        }
    };
}

impl ChannelsBot {
    pub fn available_bots() -> HashMap<String, Box<fn(&str) -> Box<dyn Bot>>> {
        bots_map!(CustomBot, GameJamBot, QuoteBot, TimerBot, VoteBot)
    }

    pub fn spawn_bot(&mut self, bot_name: &str) -> Option<String> {
        let (response, new_bot) = if self.active_bots.contains_key(bot_name) {
            (Some(format!("{} is already active", bot_name)), None)
        } else {
            match self.new_bot(bot_name) {
                Some(new_bot) => (Some(format!("{} is now active", bot_name)), Some(new_bot)),
                None => (None, None),
            }
        };
        if let Some(new_bot) = new_bot {
            println!("Spawned bot {}", bot_name);
            self.active_bots.insert(bot_name.to_owned(), new_bot);
        }
        self.save_bots().unwrap();
        response
    }

    fn disable_bot(&mut self, bot_name: &str) -> Option<String> {
        let bot = self.active_bots.remove(bot_name);
        let response = bot.map(|bot| format!("{} is no longer active", bot.name()));
        self.save_bots().unwrap();
        response
    }

    fn reset_bot(&mut self, bot_name: &str) -> Option<String> {
        self.disable_bot(bot_name);
        self.spawn_bot(bot_name)
            .map(|_| format!("{} is reset", bot_name))
    }

    fn save_bots(&self) -> std::io::Result<()> {
        let bots_config = self.bots_config().unwrap();
        let file = std::io::BufWriter::new(std::fs::File::create("config/bots_config.json")?);
        serde_json::to_writer(file, &bots_config)?;
        Ok(())
    }

    fn bots_config(&self) -> Result<BotsConfig, ()> {
        let mut bots_config = BotsConfig {
            active_bots: HashSet::with_capacity(self.active_bots.len()),
        };
        for bot_name in self.active_bots.keys() {
            bots_config.active_bots.insert(bot_name.to_owned());
        }
        Ok(bots_config)
    }

    fn new_bot(&self, bot_name: &str) -> Option<Box<dyn Bot>> {
        self.available_bots
            .get(bot_name)
            .map(|f| f(&self.channel_login))
    }

    pub fn commands() -> BotCommands<Self> {
        BotCommands {
            commands: vec![
                CommandNode::LiteralNode {
                    literals: vec!["!enable".to_owned()],
                    child_nodes: vec![CommandNode::ArgumentNode {
                        argument_type: ArgumentType::Word,
                        child_nodes: vec![CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Moderator,
                            command: Arc::new(|bot, _, mut args| {
                                let bot_name = args.remove(0);
                                let response = bot.spawn_bot(bot_name.as_str());
                                response
                            }),
                        }],
                    }],
                },
                CommandNode::LiteralNode {
                    literals: vec!["!disable".to_owned()],
                    child_nodes: vec![CommandNode::ArgumentNode {
                        argument_type: ArgumentType::Word,
                        child_nodes: vec![CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Moderator,
                            command: Arc::new(|bot, _, mut args| {
                                let bot_name = args.remove(0);
                                let response = bot.disable_bot(bot_name.as_str());
                                response
                            }),
                        }],
                    }],
                },
                CommandNode::LiteralNode {
                    literals: vec!["!reset".to_owned()],
                    child_nodes: vec![CommandNode::ArgumentNode {
                        argument_type: ArgumentType::Word,
                        child_nodes: vec![CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Moderator,
                            command: Arc::new(|bot, _, mut args| {
                                let bot_name = args.remove(0);
                                let response = bot.reset_bot(&bot_name);
                                response
                            }),
                        }],
                    }],
                },
            ],
        }
    }
}
