use super::*;

pub struct BotCommands<T> {
    pub commands: Vec<BotCommand<T>>,
}

pub enum AuthorityLevel {
    Broadcaster,
    Moderator,
    Any,
}

pub struct BotCommand<T> {
    pub name: String,
    pub authority_level: AuthorityLevel,
    pub command: fn(&mut T, String, String, String) -> Option<String>,
}

impl<T> BotCommands<T> {
    pub fn check_command(&self, message: &PrivmsgMessage) -> Option<(&BotCommand<T>, String)> {
        let mut message_text = message.message_text.clone();
        match message_text.remove(0) {
            '!' => self.find(message_text.as_str(), message).map(|command| {
                message_text.replace_range(0..command.name.len(), "");
                (command, message_text.trim().to_owned())
            }),
            _ => None,
        }
    }
    pub fn find(&self, command: &str, message: &PrivmsgMessage) -> Option<&BotCommand<T>> {
        self.commands.iter().find_map(|com| {
            if command.starts_with(&com.name) {
                if !check_authority(&com.authority_level, message) {
                    return None;
                }
                Some(com)
            } else {
                None
            }
        })
    }
}

fn check_authority(authority_level: &AuthorityLevel, message: &PrivmsgMessage) -> bool {
    match authority_level {
        AuthorityLevel::Any => true,
        AuthorityLevel::Broadcaster => check_badges(vec!["broadcaster"], message),
        AuthorityLevel::Moderator => check_badges(vec!["broadcaster", "moderator"], message),
    }
}

fn check_badges(badges: Vec<&str>, message: &PrivmsgMessage) -> bool {
    message
        .badges
        .iter()
        .any(|badge| badges.contains(&badge.name.as_str()))
}

impl ChannelsBot {
    pub fn commands() -> BotCommands<ChannelsBot> {
        BotCommands {
            commands: vec![
                BotCommand {
                    name: "enable".to_owned(),
                    authority_level: AuthorityLevel::Moderator,
                    command: |bot, _, _, args| {
                        let response = bot.spawn_bot(args.as_str());
                        bot.save_bots().unwrap();
                        response
                    },
                },
                BotCommand {
                    name: "disable".to_owned(),
                    authority_level: AuthorityLevel::Moderator,
                    command: |bot, _, _, args| {
                        let response = bot.disable_bot(args.as_str());
                        bot.save_bots().unwrap();
                        response
                    },
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
                        Some(Box::new(LDBot::new(&self.channel))),
                    )
                }
            }
            "reply" => {
                if self.bots.contains_key(bot_name) {
                    (Some("ReplyBot is already active".to_owned()), None)
                } else {
                    (
                        Some("ReplyBot is now active".to_owned()),
                        Some(Box::new(ReplyBot::new(&self.channel))),
                    )
                }
            }
            "quote" => {
                if self.bots.contains_key(bot_name) {
                    (Some("QuoteBot is already active".to_owned()), None)
                } else {
                    (
                        Some("QuoteBot is now active".to_owned()),
                        Some(Box::new(QuoteBot::new(&self.channel))),
                    )
                }
            }
            "custom" => {
                if self.bots.contains_key(bot_name) {
                    (Some("CustomBot is already active".to_owned()), None)
                } else {
                    (
                        Some("CustomBot is now active".to_owned()),
                        Some(Box::new(CustomBot::new(&self.channel))),
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
        let (response, bot) = match bot_name {
            "ludumdare" => match self.bots.remove(bot_name) {
                Some(bot) => (Some("LDBot is no longer active".to_owned()), Some(bot)),
                None => (Some("LDBot is not active at the moment".to_owned()), None),
            },
            "reply" => match self.bots.remove(bot_name) {
                Some(bot) => (Some("ReplyBot is no longer active".to_owned()), Some(bot)),
                None => (
                    Some("ReplyBot is not active at the moment".to_owned()),
                    None,
                ),
            },
            _ => (None, None),
        };
        if let Some(_) = bot {
            println!("Disabled bot {}", bot_name);
        }
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
