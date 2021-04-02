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
