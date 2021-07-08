use async_trait::async_trait;
use futures::{lock::Mutex, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_compat_02::FutureExt;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::{PrivmsgMessage, ServerMessage};
use twitch_irc::{ClientConfig, TCPTransport, TwitchIRCClient};

mod channels_bot;
mod commands;
mod custom_bot;
mod gamejam_bot;
mod quote_bot;
mod timer_bot;
mod vote_bot;

use channels_bot::{BotsConfig, ChannelsBot};
use commands::*;
use custom_bot::CustomBot;
use gamejam_bot::GameJamBot;
use quote_bot::QuoteBot;
use timer_bot::TimerBot;
use vote_bot::VoteBot;

#[tokio::main]
async fn main() {
    let login_config: LoginConfig = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("secrets/login.json").unwrap(),
    ))
    .unwrap();
    let bots_config: BotsConfig = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("config/bots_config.json").unwrap(),
    ))
    .unwrap();

    let client_config = ClientConfig::new_simple(StaticLoginCredentials::new(
        login_config.login_name.clone(),
        Some(login_config.oauth_token.clone()),
    ));

    let (mut incoming_messages, client) =
        async { TwitchIRCClient::<TCPTransport, StaticLoginCredentials>::new(client_config) }
            .compat()
            .await;

    let channels_bot = Arc::new(Mutex::new(ChannelsBot::new(&login_config, &bots_config)));

    let bot = Arc::clone(&channels_bot);
    let client_clone = client.clone();
    let message_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.next().await {
            bot.lock()
                .await
                .handle_message(&client_clone, message)
                .await;
        }
    });

    let bot = Arc::clone(&channels_bot);
    let client_clone = client.clone();
    let update_handle = tokio::spawn(async move {
        const FIXED_DELTA_TIME: f32 = 1.0;
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs_f32(FIXED_DELTA_TIME));
        loop {
            interval.tick().await;
            bot.lock()
                .await
                .update(&client_clone, FIXED_DELTA_TIME)
                .await;
        }
    });

    client.join(login_config.channel_login);

    message_handle.await.unwrap();
    update_handle.await.unwrap();
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

    async fn update(
        &mut self,
        _client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        _delta_time: f32,
    ) {
    }

    fn update_status(&self, status_text: &str) {
        let path = format!("status/{}.txt", self.name());
        std::fs::write(path, status_text).expect("Could not update bot status");
    }
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
