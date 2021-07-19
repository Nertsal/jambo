use async_trait::async_trait;
use futures::{lock::Mutex, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_compat_02::FutureExt;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::{PrivmsgMessage, ServerMessage};
use twitch_irc::{ClientConfig, TCPTransport, TwitchIRCClient};

mod bot;
mod channels_bot;
mod commands;
mod completion;
mod custom_bot;
mod gamejam_bot;
mod quote_bot;
mod timer_bot;
mod vote_bot;

use bot::*;
use channels_bot::{ActiveBots, ChannelsBot};
use commands::*;
use completion::*;
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
    let active_bots: ActiveBots = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("config/active_bots.json").unwrap(),
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

    let cli = Arc::new(linefeed::Interface::new("nertsal-bot").unwrap());
    let channels_bot = ChannelsBot::new(&cli, &login_config, &active_bots);
    let completer = Arc::new(CommandCompleter {
        completion_tree: channels_bot.get_completion_tree(),
    });
    cli.set_completer(completer);
    let channels_bot = Arc::new(Mutex::new(channels_bot));

    let bot = Arc::clone(&channels_bot);
    let client_clone = client.clone();
    let message_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.next().await {
            let mut bot_lock = bot.lock().await;
            bot_lock.handle_message(&client_clone, message).await;
            if bot_lock.queue_shutdown {
                break;
            }
        }
        bot.lock().await.log(LogType::Info, "Chat handle shut down");
    });

    let bot = Arc::clone(&channels_bot);
    let client_clone = client.clone();
    let update_handle = tokio::spawn(async move {
        const FIXED_DELTA_TIME: f32 = 1.0;
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs_f32(FIXED_DELTA_TIME));
        loop {
            interval.tick().await;
            let mut bot_lock = bot.lock().await;
            bot_lock.update(&client_clone, FIXED_DELTA_TIME).await;
            if bot_lock.queue_shutdown {
                break;
            }
        }
        bot.lock()
            .await
            .log(LogType::Info, "Update handle shut down");
    });

    let bot = Arc::clone(&channels_bot);
    let client_clone = client.clone();
    let console_handle = tokio::spawn(async move {
        cli.set_prompt("> ").unwrap();
        while let linefeed::ReadResult::Input(input) = cli.read_line().unwrap() {
            let mut bot_lock = bot.lock().await;
            bot_lock
                .handle_command_message(
                    &client_clone,
                    CommandMessage {
                        sender_name: "Admin".to_owned(),
                        message_text: input.clone(),
                        authority_level: AuthorityLevel::Broadcaster,
                        origin: MessageOrigin::Console,
                    },
                )
                .await;
            if bot_lock.queue_shutdown {
                break;
            }
        }
        bot.lock()
            .await
            .log(LogType::Info, "Console handle shut down");
    });

    client.join(login_config.channel_login.clone());

    message_handle.await.unwrap();
    update_handle.await.unwrap();
    console_handle.await.unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct LoginConfig {
    login_name: String,
    oauth_token: String,
    channel_login: String,
}
