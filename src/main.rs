use async_trait::async_trait;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Instant};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::{PrivmsgMessage, ServerMessage};
use twitch_irc::{ClientConfig, TCPTransport, TwitchIRCClient};

mod commands;
mod custom_bot;
mod id;
mod ld_bot;
mod quote_bot;
mod reply_bot;

use commands::*;
use custom_bot::CustomBot;
use id::*;
use ld_bot::LDBot;
use quote_bot::QuoteBot;
use reply_bot::ReplyBot;

#[tokio::main]
async fn main() {
    let nertsalbot_config: Config = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("config/nertsalbot.json").unwrap(),
    ))
    .unwrap();
    let bots_config: BotsConfig = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("config/bots-config.json").unwrap(),
    ))
    .unwrap();

    let client_config = ClientConfig::new_simple(StaticLoginCredentials::new(
        nertsalbot_config.login_name.clone(),
        Some(nertsalbot_config.oauth_token.clone()),
    ));
    let (mut incoming_messages, client) =
        TwitchIRCClient::<TCPTransport, StaticLoginCredentials>::new(client_config);

    let mut channels_bot = ChannelsBot::new(&nertsalbot_config, &bots_config);

    let client_clone = client.clone();
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.next().await {
            channels_bot.handle_message(&client_clone, message).await;
        }
    });

    client.join(nertsalbot_config.channel);

    join_handle.await.unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    login_name: String,
    oauth_token: String,
    channel: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct BotsConfig {
    ludumdare: bool,
    reply: bool,
    quote: bool,
    custom: bool,
}

struct ChannelsBot {
    channel: String,
    commands: BotCommands<ChannelsBot>,
    bots: HashMap<String, Box<dyn Bot>>,
}

impl ChannelsBot {
    fn new(config: &Config, bots_config: &BotsConfig) -> Self {
        let mut bot = Self {
            channel: config.channel.clone(),
            commands: Self::commands(),
            bots: HashMap::new(),
        };
        if bots_config.ludumdare {
            bot.spawn_bot("ludumdare");
        }
        if bots_config.reply {
            bot.spawn_bot("reply");
        }
        if bots_config.quote {
            bot.spawn_bot("quote");
        }
        if bots_config.custom {
            bot.spawn_bot("custom");
        }
        bot
    }
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: ServerMessage,
    ) {
        match &message {
            ServerMessage::Join(message) => {
                println!("Joined: {}", message.channel_login);
            }
            ServerMessage::Notice(message) => {
                if message.message_text == "Login authentication failed" {
                    panic!("Login authentication failed.");
                }
            }
            ServerMessage::Privmsg(message) => {
                println!(
                    "Got a message in channel {} from {}: {}",
                    message.channel_login, message.sender.name, message.message_text
                );
                check_command(self, client, self.channel.clone(), message).await;
            }
            _ => (),
        }
        for bot in self.bots.values_mut() {
            bot.handle_message(client, &message).await;
        }
    }
}

#[async_trait]
pub trait Bot: Send + Sync {
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &ServerMessage,
    );
}

pub trait CommandBot<T> {
    fn commands(&self) -> &BotCommands<T>;
}

impl CommandBot<ChannelsBot> for ChannelsBot {
    fn commands(&self) -> &BotCommands<ChannelsBot> {
        &self.commands
    }
}

async fn check_command<T: CommandBot<T>>(
    bot: &mut T,
    client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
    channel_login: String,
    message: &PrivmsgMessage,
) {
    if let Some((command, args)) = bot.commands().check_command(message) {
        let command_name = command.name.clone();
        if let Some(command_reply) =
            (command.command)(bot, message.sender.name.clone(), command_name, args)
        {
            send_message(client, channel_login, command_reply).await;
        }
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
