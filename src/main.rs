use futures::{lock::Mutex, prelude::*};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
use tokio_compat_02::FutureExt;
use twitch_irc::{login::StaticLoginCredentials, ClientConfig};

use twitch_bot::prelude::*;

#[tokio::main]
async fn main() {
    let login_config: LoginConfig = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("secrets/login.json").expect("Missing secrets/login.json"),
    ))
    .expect("Failed to parse secrets/login.json");
    let active_bots: ActiveBots = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("config/active_bots.json").expect("Missing config/active_bots.json"),
    ))
    .expect("Failed to parse config/active_bots.json");

    let client_config = ClientConfig::new_simple(StaticLoginCredentials::new(
        login_config.login_name.clone(),
        Some(login_config.oauth_token.clone()),
    ));
    let channel_login = login_config.channel_login;

    let (mut incoming_messages, client) = async { TwitchClient::new(client_config) }.compat().await;

    let cli = Arc::new(linefeed::Interface::new("nertsal-bot").unwrap());
    let channels_bot = ChannelsBot::new(&cli, active_bots);
    // let completer = Arc::new(CommandCompleter {
    //     completion_tree: channels_bot.get_completion_tree(),
    // });
    // cli.set_completer(completer);
    let channels_bot = Arc::new(Mutex::new(channels_bot));

    let bot = Arc::clone(&channels_bot);
    let client_clone = client.clone();
    let message_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.next().await {
            let mut bot_lock = bot.lock().await;
            bot_lock.handle_server_message(&client_clone, message).await;
            // if bot_lock.queue_shutdown {
            //     break;
            // }
        }
        // bot.lock().await.log(LogType::Info, "Chat handle shut down");
    });

    let bot = Arc::clone(&channels_bot);
    let client_clone = client.clone();
    let channel_login_clone = channel_login.clone();
    let update_handle = tokio::spawn(async move {
        const FIXED_DELTA_TIME: f32 = 1.0;
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs_f32(FIXED_DELTA_TIME));
        loop {
            interval.tick().await;
            let mut bot_lock = bot.lock().await;
            bot_lock
                .update(&client_clone, &channel_login_clone, FIXED_DELTA_TIME)
                .await;
            // if bot_lock.queue_shutdown {
            //     break;
            // }
        }
        // bot.lock()
        //     .await
        //     .log(LogType::Info, "Update handle shut down");
    });

    let bot = Arc::clone(&channels_bot);
    let client_clone = client.clone();
    let channel_login_clone = channel_login.clone();
    let console_handle = tokio::spawn(async move {
        cli.set_prompt("> ").unwrap();
        while let linefeed::ReadResult::Input(input) = cli.read_line().unwrap() {
            let mut bot_lock = bot.lock().await;
            bot_lock
                .handle_command_message(
                    &client_clone,
                    &channel_login_clone,
                    CommandMessage {
                        sender: Sender {
                            name: "Admin".to_owned(),
                            origin: MessageOrigin::Console,
                        },
                        message_text: input.clone(),
                        authority_level: AuthorityLevel::Broadcaster as usize,
                    },
                )
                .await;
            // if bot_lock.queue_shutdown {
            //     break;
            // }
        }
        // bot.lock()
        //     .await
        //     .log(LogType::Info, "Console handle shut down");
    });

    client.join(channel_login);

    message_handle.await.unwrap();
    update_handle.await.unwrap();
    console_handle.await.unwrap();

    // channels_bot
    //     .lock()
    //     .await
    //     .log(LogType::Info, "Shut down succefully");
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginConfig {
    pub login_name: String,
    pub oauth_token: String,
    pub channel_login: String,
}

pub type BotName = String;
pub type ChannelLogin = String;
pub type ActiveBots = HashSet<BotName>;

pub struct ChannelsBot {}

impl ChannelsBot {
    pub fn new(
        cli: &Arc<linefeed::Interface<impl linefeed::Terminal>>,
        active_bots: ActiveBots,
    ) -> Self {
        Self {}
    }

    pub async fn handle_server_message(&mut self, client: &TwitchClient, message: ServerMessage) {}

    pub async fn handle_command_message(
        &mut self,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: CommandMessage,
    ) {
    }

    pub async fn update(&mut self, client: &TwitchClient, channel: &ChannelLogin, delta_time: f32) {
    }
}
