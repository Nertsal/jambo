use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum AuthorityLevel {
    Viewer,
    Moderator,
    Broadcaster,
}

pub type TwitchClient = twitch_irc::TwitchIRCClient<
    twitch_irc::TCPTransport,
    twitch_irc::login::StaticLoginCredentials,
>;

pub type CommandMessage = nertsal_commands::CommandMessage<Sender>;

pub mod prelude {
    pub use crate::{AuthorityLevel, CommandMessage, MessageOrigin, Sender, TwitchClient};
    pub use async_trait::async_trait;
    pub use futures;
    pub use nertsal_commands::*;
    pub use serde::{self, Deserialize, Serialize};
    pub use serde_json;
    pub use tokio;
    pub use tokio_compat_02;
    pub use twitch_irc::{self, message::ServerMessage};
}
