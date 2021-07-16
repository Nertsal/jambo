use twitch_irc::message::{Badge, PrivmsgMessage};

pub struct CommandMessage {
    pub sender_name: String,
    pub message_text: String,
    pub badges: Vec<Badge>,
}

impl From<&PrivmsgMessage> for CommandMessage {
    fn from(message: &PrivmsgMessage) -> Self {
        Self {
            sender_name: message.sender.name.clone(),
            message_text: message.message_text.clone(),
            badges: message.badges.clone(),
        }
    }
}
