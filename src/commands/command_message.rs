use twitch_irc::message::PrivmsgMessage;

use super::AuthorityLevel;

pub struct CommandMessage {
    pub sender_name: String,
    pub message_text: String,
    pub authority_level: AuthorityLevel,
    pub origin: MessageOrigin,
}

impl From<&PrivmsgMessage> for CommandMessage {
    fn from(message: &PrivmsgMessage) -> Self {
        Self {
            sender_name: message.sender.name.clone(),
            message_text: message.message_text.clone(),
            authority_level: AuthorityLevel::from_badges(&message.badges),
            origin: MessageOrigin::Chat,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MessageOrigin {
    Chat,
    Console,
}
