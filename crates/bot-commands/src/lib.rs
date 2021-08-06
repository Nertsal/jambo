pub use nertsal_commands::{CommandMessage, Commands, Response};
use std::sync::Arc;
use twitch_irc::{
    login::StaticLoginCredentials, message::PrivmsgMessage, TCPTransport, TwitchIRCClient,
};

mod command_bot;
mod log;

pub use command_bot::*;
pub use log::*;

pub type TwitchClient = TwitchIRCClient<TCPTransport, StaticLoginCredentials>;
pub type CLI = Arc<linefeed::Interface<linefeed::DefaultTerminal>>;

pub struct Sender {
    pub name: String,
    pub origin: MessageOrigin,
}

#[derive(Clone, Copy, Debug)]
pub enum MessageOrigin {
    Chat,
    Console,
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub enum AuthorityLevel {
    Viewer = 0,
    Moderator = 1,
    Broadcaster = 2,
}

impl AuthorityLevel {
    pub fn from_badges(badges: &Vec<twitch_irc::message::Badge>) -> Self {
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

pub fn private_to_command_message(message: &PrivmsgMessage) -> CommandMessage<Sender> {
    CommandMessage {
        sender: Sender {
            name: message.sender.name.clone(),
            origin: MessageOrigin::Chat,
        },
        message_text: message.message_text.clone(),
        authority_level: AuthorityLevel::from_badges(&message.badges) as usize,
    }
}
