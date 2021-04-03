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
                    literal: "enable".to_owned(),
                    child_nodes: vec![CommandNode::ArgumentNode {
                        argument_type: ArgumentType::Word,
                        child_node: Box::new(CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Broadcaster,
                            command: Arc::new(|bot, _, mut args| {
                                let bot_name = args.remove(0);
                                let response = bot.spawn_bot(bot_name.as_str());
                                bot.save_bots().unwrap();
                                response
                            }),
                        }),
                    }],
                },
                CommandNode::LiteralNode {
                    literal: "disable".to_owned(),
                    child_nodes: vec![CommandNode::ArgumentNode {
                        argument_type: ArgumentType::Word,
                        child_node: Box::new(CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Broadcaster,
                            command: Arc::new(|bot, _, mut args| {
                                let bot_name = args.remove(0);
                                let response = bot.disable_bot(bot_name.as_str());
                                bot.save_bots().unwrap();
                                response
                            }),
                        }),
                    }],
                },
            ],
        }
    }
    pub fn spawn_bot(&mut self, bot_name: &str) -> Option<String> {
        let (response, new_bot): (Option<String>, Option<Box<dyn Bot>>) = match bot_name {
            "ludumdare" => {
                if self.bots.contains_key(bot_name) {
                    (Some("LDBot is already active".to_owned()), None)
                } else {
                    (
                        Some("LDBot is now active".to_owned()),
                        Some(Box::new(LDBot::new(&self.channel_login))),
                    )
                }
            }
            "reply" => {
                if self.bots.contains_key(bot_name) {
                    (Some("ReplyBot is already active".to_owned()), None)
                } else {
                    (
                        Some("ReplyBot is now active".to_owned()),
                        Some(Box::new(ReplyBot::new(&self.channel_login))),
                    )
                }
            }
            "quote" => {
                if self.bots.contains_key(bot_name) {
                    (Some("QuoteBot is already active".to_owned()), None)
                } else {
                    (
                        Some("QuoteBot is now active".to_owned()),
                        Some(Box::new(QuoteBot::new(&self.channel_login))),
                    )
                }
            }
            "custom" => {
                if self.bots.contains_key(bot_name) {
                    (Some("CustomBot is already active".to_owned()), None)
                } else {
                    (
                        Some("CustomBot is now active".to_owned()),
                        Some(Box::new(CustomBot::new(&self.channel_login))),
                    )
                }
            }
            _ => (None, None),
        };
        if let Some(new_bot) = new_bot {
            println!("Spawned bot {}", bot_name);
            self.bots.insert(bot_name.to_owned(), new_bot);
        }
        response
    }
    fn disable_bot(&mut self, bot_name: &str) -> Option<String> {
        let bot = self.bots.remove(bot_name);
        let response = bot.map(|bot| format!("{} is not longer active", bot.name()));
        response
    }
    fn save_bots(&self) -> std::io::Result<()> {
        let bots_config = self.bots_config().unwrap();
        let file = std::io::BufWriter::new(std::fs::File::create("config/bots-config.json")?);
        serde_json::to_writer(file, &bots_config)?;
        Ok(())
    }
    fn bots_config(&self) -> Result<BotsConfig, ()> {
        let mut bots_config = BotsConfig {
            ludumdare: false,
            reply: false,
            quote: false,
            custom: false,
        };
        for bot_name in self.bots.keys() {
            match bot_name.as_str() {
                "ludumdare" => bots_config.ludumdare = true,
                "reply" => bots_config.reply = true,
                "quote" => bots_config.quote = true,
                "custom" => bots_config.custom = true,
                _ => return Err(()),
            }
        }
        Ok(bots_config)
    }
}
