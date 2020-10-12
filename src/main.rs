use futures::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::{PrivmsgMessage, ServerMessage};
use twitch_irc::{ClientConfig, TCPTransport, TwitchIRCClient};

mod ld_bot;

use ld_bot::*;

#[tokio::main]
async fn main() {
    let bot_config: Config = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("config.json").unwrap(),
    ))
    .unwrap();

    let config = ClientConfig::new_simple(StaticLoginCredentials::new(
        bot_config.login_name.clone(),
        Some(bot_config.oauth_token.clone()),
    ));
    let (mut incoming_messages, client) =
        TwitchIRCClient::<TCPTransport, StaticLoginCredentials>::new(config);

    let channels = bot_config.channels.clone();
    let mut channels_bot = ChannelsBot {
        bots: {
            let mut map = HashMap::new();
            for channel in &channels {
                let mut save_file = channel.clone();
                save_file.push_str("-nertsalbot.json");
                map.insert(channel.clone(), LDBot::new(&bot_config, save_file));
            }
            map
        },
    };

    // first thing you should do: start consuming incoming messages,
    // otherwise they will back up.
    let client_clone = client.clone();
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.next().await {
            channels_bot.handle_message(&client_clone, message).await;
        }
    });

    // join a channel
    for channel in channels {
        client.join(channel);
    }

    // keep the tokio executor alive.
    // If you return instead of waiting the background task will exit.
    join_handle.await.unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    login_name: String,
    oauth_token: String,
    channels: Vec<String>,
    authorities: HashSet<String>,
    response_time_limit: Option<u64>,
}

struct ChannelsBot {
    bots: HashMap<String, LDBot>,
}

impl ChannelsBot {
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: ServerMessage,
    ) {
        for (channel, bot) in &mut self.bots {
            bot.handle_message(channel, client, &message).await;
        }
    }
}
