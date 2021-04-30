use async_trait::async_trait;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Instant};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::{PrivmsgMessage, ServerMessage};
use twitch_irc::{ClientConfig, TCPTransport, TwitchIRCClient};

mod channels_bot;
mod commands;
mod custom_bot;
mod gamejam_bot;
mod id;
mod quote_bot;
mod reply_bot;

use channels_bot::{BotsConfig, ChannelsBot};
use commands::*;
use custom_bot::CustomBot;
use gamejam_bot::GameJamBot;
use id::*;
use quote_bot::QuoteBot;
use reply_bot::ReplyBot;

#[tokio::main]
async fn main() {
    let login_config: LoginConfig = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("secrets/login.json").unwrap(),
    ))
    .unwrap();
    let bots_config: BotsConfig = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("config/bots-config.json").unwrap(),
    ))
    .unwrap();

    let client_config = ClientConfig::new_simple(StaticLoginCredentials::new(
        login_config.login_name.clone(),
        Some(login_config.oauth_token.clone()),
    ));
    let (mut incoming_messages, client) =
        TwitchIRCClient::<TCPTransport, StaticLoginCredentials>::new(client_config);

    let mut channels_bot = ChannelsBot::new(&login_config, &bots_config);

    let client_clone = client.clone();
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.next().await {
            channels_bot.handle_message(&client_clone, message).await;
        }
    });

    client.join(login_config.channel_login);

    join_handle.await.unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct LoginConfig {
    login_name: String,
    oauth_token: String,
    channel_login: String,
}

#[async_trait]
pub trait Bot: Send + Sync {
    fn name(&self) -> &str;
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &ServerMessage,
    );
}

async fn send_message(
    client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
    channel_login: String,
    message: String,
) {
    println!(
        "Sending a message to channel {}: {}",
        channel_login, message
    );
    client.say(channel_login, message).await.unwrap();
}
