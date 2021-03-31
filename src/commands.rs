use super::*;
use std::collections::HashSet;

pub struct BotCommands<T> {
    pub authorities: HashSet<String>,
    pub commands: Vec<BotCommand<T>>,
}

pub struct BotCommand<T> {
    pub name: String,
    pub authorities_required: bool,
    pub command: fn(&mut T, String, String) -> Option<String>,
}

impl<T> BotCommands<T> {
    pub fn check_command(&self, message: &PrivmsgMessage) -> Option<(&BotCommand<T>, String)> {
        let mut message_text = message.message_text.clone();
        match message_text.remove(0) {
            '!' => {
                let mut args = message_text.split_whitespace();
                if let Some(command) = args.next() {
                    self.find(command, &message.sender.login)
                        .map(|command| (command, args.collect()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn find(&self, command: &str, sender_login: &String) -> Option<&BotCommand<T>> {
        self.commands.iter().find_map(|com| {
            if com.name == command {
                if com.authorities_required && !self.authorities.contains(sender_login) {
                    return None;
                }
                Some(com)
            } else {
                None
            }
        })
    }
}

impl ChannelsBot {
    pub fn commands(config: &Config) -> BotCommands<ChannelsBot> {
        BotCommands {
            authorities: config.authorities.clone(),
            commands: vec![
                BotCommand {
                    name: "enable".to_owned(),
                    authorities_required: true,
                    command: |bot, _, args| bot.spawn_bot(args.as_str()),
                },
                BotCommand {
                    name: "disable".to_owned(),
                    authorities_required: true,
                    command: |bot, _, args| bot.disable_bot(args.as_str()),
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
}
