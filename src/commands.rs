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
            commands: vec![BotCommand {
                name: "enable".to_owned(),
                authorities_required: true,
                command: |bot, _, args| match args.as_str() {
                    "ludumdare" => {
                        if bot.active_bots.ludum_dare {
                            Some("LDBot is already active".to_owned())
                        } else {
                            bot.active_bots.ludum_dare = true;
                            bot.bots.push(Box::new(LDBot::new(&bot.channel)));
                            Some("LDBot is now active".to_owned())
                        }
                    }
                    "reply" => {
                        if bot.active_bots.reply {
                            Some("ReplyBot is already active".to_owned())
                        } else {
                            bot.active_bots.reply = true;
                            bot.bots.push(Box::new(ReplyBot::new(&bot.channel)));
                            Some("ReplyBot is now active".to_owned())
                        }
                    }
                    _ => None,
                },
            }],
        }
    }
}
