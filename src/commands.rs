use std::collections::HashSet;

use super::*;

pub struct BotCommands<T: Bot> {
    pub authorities: HashSet<String>,
    pub commands: Vec<BotCommand<T>>,
}

pub struct BotCommand<T: Bot> {
    pub name: String,
    pub authorities_required: bool,
    pub command: fn(&mut T, String, String) -> Option<String>,
}

impl<T: Bot> BotCommands<T> {
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
