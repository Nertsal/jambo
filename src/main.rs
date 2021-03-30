use async_trait::async_trait;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::{PrivmsgMessage, ServerMessage};
use twitch_irc::{ClientConfig, TCPTransport, TwitchIRCClient};

mod ld_bot;

use ld_bot::LDBot;

#[tokio::main]
async fn main() {
    let bot_config: Config = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("bots-config.json").unwrap(),
    ))
    .unwrap();

    let config = ClientConfig::new_simple(StaticLoginCredentials::new(
        bot_config.login_name.clone(),
        Some(bot_config.oauth_token.clone()),
    ));
    let (mut incoming_messages, client) =
        TwitchIRCClient::<TCPTransport, StaticLoginCredentials>::new(config);

    let mut channels_bot = ChannelsBot::new(&bot_config);

    let client_clone = client.clone();
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.next().await {
            channels_bot.handle_message(&client_clone, message).await;
        }
    });

    client.join(bot_config.channel);

    join_handle.await.unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    login_name: String,
    oauth_token: String,
    channel: String,
}

struct ChannelsBot {
    bots: Vec<Box<dyn Bot>>,
}

impl ChannelsBot {
    fn new(config: &Config) -> Self {
        Self {
            bots: vec![Box::new(LDBot::new(&config.channel))],
        }
    }
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: ServerMessage,
    ) {
        for bot in &mut self.bots {
            bot.handle_message(client, &message).await;
        }
    }
}

#[async_trait]
pub trait Bot: Send {
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &ServerMessage,
    );
}
