use serde::{Deserialize, Serialize};
use twitch_irc::{
    login::StaticLoginCredentials, message::PrivmsgMessage, TCPTransport, TwitchIRCClient,
};

pub type TwitchClient = TwitchIRCClient<TCPTransport, StaticLoginCredentials>;
pub type CommandMessage = nertsal_commands::CommandMessage<Sender>;
pub type Commands<T> = nertsal_commands::Commands<T, Sender>;
pub type CommandBuilder<T> = nertsal_commands::CommandBuilder<T, Sender>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sender {
    pub name: String,
    pub origin: MessageOrigin,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum MessageOrigin {
    Console,
    Twitch,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthorityLevel {
    Viewer = 0,
    Moderator = 1,
    Broadcaster = 2,
    Server = 3,
}

impl AuthorityLevel {
    pub fn from_badges(badges: &[twitch_irc::message::Badge]) -> Self {
        badges
            .iter()
            .fold(AuthorityLevel::Viewer, |authority_level, badge| {
                authority_level.max(AuthorityLevel::from_badge(badge))
            })
    }

    pub fn from_badge(badge: &twitch_irc::message::Badge) -> Self {
        match badge.name.as_str() {
            "broadcaster" => AuthorityLevel::Broadcaster,
            "moderator" => AuthorityLevel::Moderator,
            _ => AuthorityLevel::Viewer,
        }
    }
}

pub fn private_to_command_message(message: &PrivmsgMessage) -> CommandMessage {
    CommandMessage {
        sender: Sender {
            name: message.sender.name.clone(),
            origin: MessageOrigin::Twitch,
        },
        message_text: message.message_text.clone(),
        authority_level: AuthorityLevel::from_badges(&message.badges) as usize,
    }
}
