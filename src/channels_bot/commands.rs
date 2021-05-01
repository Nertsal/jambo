use std::sync::Arc;

use super::*;

impl CommandBot<Self> for ChannelsBot {
    fn get_commands(&self) -> &BotCommands<Self> {
        &self.commands
    }
}

impl ChannelsBot {
    pub fn commands() -> BotCommands<Self> {
        BotCommands {
            commands: vec![
                CommandNode::LiteralNode {
                    literal: "!enable".to_owned(),
                    child_nodes: vec![CommandNode::ArgumentNode {
                        argument_type: ArgumentType::Word,
                        child_node: Box::new(CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Moderator,
                            command: Arc::new(|bot, _, mut args| {
                                let bot_name = args.remove(0);
                                let response = bot.spawn_bot(bot_name.as_str());
                                response
                            }),
                        }),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "!disable".to_owned(),
                    child_nodes: vec![CommandNode::ArgumentNode {
                        argument_type: ArgumentType::Word,
                        child_node: Box::new(CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Moderator,
                            command: Arc::new(|bot, _, mut args| {
                                let bot_name = args.remove(0);
                                let response = bot.disable_bot(bot_name.as_str());
                                response
                            }),
                        }),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "!reset".to_owned(),
                    child_nodes: vec![CommandNode::ArgumentNode {
                        argument_type: ArgumentType::Word,
                        child_node: Box::new(CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Moderator,
                            command: Arc::new(|bot, _, mut args| {
                                let bot_name = args.remove(0);
                                let response = bot.reset_bot(&bot_name);
                                response
                            }),
                        }),
                    }],
                },
            ],
        }
    }
    pub fn spawn_bot(&mut self, bot_name: &str) -> Option<String> {
        let (response, new_bot) = if self.bots.contains_key(bot_name) {
            (Some(format!("{} is already active", bot_name)), None)
        } else {
            match self.new_bot(bot_name) {
                Some(new_bot) => (Some(format!("{} is now active", bot_name)), Some(new_bot)),
                None => (None, None),
            }
        };
        if let Some(new_bot) = new_bot {
            println!("Spawned bot {}", bot_name);
            self.bots.insert(bot_name.to_owned(), new_bot);
        }
        self.save_bots().unwrap();
        response
    }
    fn disable_bot(&mut self, bot_name: &str) -> Option<String> {
        let bot = self.bots.remove(bot_name);
        let response = bot.map(|bot| format!("{} is no longer active", bot.name()));
        self.save_bots().unwrap();
        response
    }
    fn reset_bot(&mut self, bot_name: &str) -> Option<String> {
        self.disable_bot(bot_name);
        self.spawn_bot(bot_name);
        Some(format!("{} is reset", bot_name))
    }
    fn save_bots(&self) -> std::io::Result<()> {
        let bots_config = self.bots_config().unwrap();
        let file = std::io::BufWriter::new(std::fs::File::create("config/bots_config.json")?);
        serde_json::to_writer(file, &bots_config)?;
        Ok(())
    }
    fn bots_config(&self) -> Result<BotsConfig, ()> {
        let mut bots_config = BotsConfig {
            gamejam: false,
            reply: false,
            quote: false,
            custom: false,
        };
        for bot_name in self.bots.keys() {
            if bot_name == GameJamBot::name() {
                bots_config.gamejam = true;
            } else if bot_name == ReplyBot::name() {
                bots_config.reply = true;
            } else if bot_name == QuoteBot::name() {
                bots_config.quote = true;
            } else if bot_name == CustomBot::name() {
                bots_config.custom = true;
            } else {
                return Err(());
            }
        }
        Ok(bots_config)
    }
    fn new_bot(&self, bot_name: &str) -> Option<Box<dyn Bot>> {
        if bot_name == GameJamBot::name() {
            Some(Box::new(GameJamBot::new(&self.channel_login)))
        } else if bot_name == ReplyBot::name() {
            Some(Box::new(ReplyBot::new(&self.channel_login)))
        } else if bot_name == QuoteBot::name() {
            Some(Box::new(QuoteBot::new(&self.channel_login)))
        } else if bot_name == CustomBot::name() {
            Some(Box::new(CustomBot::new(&self.channel_login)))
        } else {
            None
        }
    }
}
