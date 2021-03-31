use async_trait::async_trait;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, time::Instant};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::{PrivmsgMessage, ServerMessage};
use twitch_irc::{ClientConfig, TCPTransport, TwitchIRCClient};

mod commands;
mod ld_bot;
mod reply_bot;

use commands::*;
use ld_bot::LDBot;
use reply_bot::ReplyBot;

#[tokio::main]
async fn main() {
    let bot_config: Config = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("config/bots-config.json").unwrap(),
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
    bots: BotsConfig,
    authorities: HashSet<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct BotsConfig {
    ludum_dare: bool,
    reply: bool,
}

struct ChannelsBot {
    channel: String,
    commands: BotCommands<ChannelsBot>,
    active_bots: BotsConfig,
    bots: Vec<Box<dyn Bot>>,
}

impl ChannelsBot {
    fn new(config: &Config) -> Self {
        let mut bots: Vec<Box<dyn Bot>> = Vec::new();
        if config.bots.ludum_dare {
            bots.push(Box::new(LDBot::new(&config.channel)));
        }
        if config.bots.reply {
            bots.push(Box::new(ReplyBot::new(&config.channel)));
        }
        Self {
            channel: config.channel.clone(),
            commands: Self::commands(config),
            active_bots: config.bots.clone(),
            bots,
        }
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
        for bot in &mut self.bots {
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
        if let Some(command_reply) = (command.command)(
            bot,
            message.sender.name.clone(),
            args,
        ) {
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
