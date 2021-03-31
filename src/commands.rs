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
    pub command: fn(&mut T, String, String) -> Option<String>,
}

impl<T> BotCommands<T> {
    pub fn check_command(&self, message: &PrivmsgMessage) -> Option<(&BotCommand<T>, String)> {
        let mut message_text = message.message_text.clone();
        match message_text.remove(0) {
            '!' => {
                let mut args = message_text.split_whitespace();
                if let Some(command) = args.next() {
                    self.find(command, message)
                        .map(|command| (command, args.collect()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn find(&self, command: &str, message: &PrivmsgMessage) -> Option<&BotCommand<T>> {
        self.commands.iter().find_map(|com| {
            if com.name == command {
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
    println!("{:?}", message.badges);
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
                    command: |bot, _, args| bot.spawn_bot(args.as_str()),
                },
                BotCommand {
                    name: "disable".to_owned(),
                    authority_level: AuthorityLevel::Moderator,
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
