use futures::{lock::Mutex, prelude::*};
use linefeed::Completer;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Display, sync::Arc};
use tokio_compat_02::FutureExt;
use twitch_irc::{login::StaticLoginCredentials, ClientConfig};

use twitch_bot::prelude::*;

mod bots;
mod main_bot;

use main_bot::*;

const CONSOLE_PREFIX_LENGTH: usize = 7;

#[tokio::main]
async fn main() {
    // Load config
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

    // Connect to Twitch
    let (mut incoming_messages, client) = async { TwitchClient::new(client_config) }.compat().await;

    // Setup CLI
    let cli = Arc::new(linefeed::Interface::new("nertsal-bot").unwrap());
    let main_bot = MainBot::new(&cli, active_bots);
    let main_bot = Arc::new(MutexBot::new(main_bot));
    let completer = main_bot.clone();
    cli.set_completer(completer);

    // Initialize twitch handle
    let bot = Arc::clone(&main_bot);
    let client_clone = client.clone();
    let (message_handle, message_abort) = futures::future::abortable(tokio::spawn(async move {
        while let Some(message) = incoming_messages.next().await {
            let mut bot_lock = bot.lock().await;
            bot_lock.handle_server_message(&client_clone, message).await;
            // if bot_lock.queue_shutdown {
            //     break;
            // }
        }
        bot.lock().await.log(LogType::Info, "Chat handle shut down");
    }));

    // Initialize CLI handle
    let bot = Arc::clone(&main_bot);
    let client_clone = client.clone();
    let channel_login_clone = channel_login.clone();
    let (console_handle, console_abort) = futures::future::abortable(tokio::spawn(async move {
        cli.set_prompt(&format!("{:w$} > ", " ", w = CONSOLE_PREFIX_LENGTH))
            .unwrap();
        while let linefeed::ReadResult::Input(input) = cli.read_line().unwrap() {
            let mut bot_lock = bot.lock().await;
            bot_lock
                .handle_message(
                    &client_clone,
                    &channel_login_clone,
                    &CommandMessage {
                        sender: Sender {
                            name: "Admin".to_owned(),
                            origin: MessageOrigin::Console,
                        },
                        message_text: input.clone(),
                        authority_level: AuthorityLevel::Broadcaster as usize,
                    },
                )
                .await;

            if bot_lock.queue_shutdown {
                break;
            }
        }
    }));

    // Initialize update handle
    let bot = Arc::clone(&main_bot);
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

            if bot_lock.queue_shutdown {
                console_abort.abort();
                message_abort.abort();
                break;
            }
        }
    });

    // Wait for all threads to finish
    client.join(channel_login);
    update_handle.await.unwrap();
    message_handle.await.unwrap_err();
    console_handle.await.unwrap_err();

    main_bot
        .lock()
        .await
        .log(LogType::Info, "Shut down succefully");
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginConfig {
    pub login_name: String,
    pub oauth_token: String,
    pub channel_login: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum LogType {
    Info,
    Warn,
    Error,
    Chat,
    Send,
    Console,
}

pub type BotName = String;
pub type ChannelLogin = String;
pub type ActiveBots = HashSet<BotName>;
pub type Prompter<'a, 'b> = linefeed::Prompter<'a, 'b, linefeed::DefaultTerminal>;
pub type Cli = Arc<linefeed::Interface<linefeed::DefaultTerminal>>;

pub fn log(cli: &Cli, log_type: LogType, message: &str) {
    let mut writer = cli.lock_writer_erase().unwrap();
    writeln!(writer, "{} {}", log_type, message).unwrap();
}

pub async fn send_message(cli: &Cli, client: &TwitchClient, channel: String, message: String) {
    log(cli, LogType::Send, &format!("{}: {}", channel, message));
    client.say(channel, message).await.unwrap();
}

impl Display for LogType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::*;
        let w = CONSOLE_PREFIX_LENGTH;
        match &self {
            LogType::Info => write!(f, "{:>w$} >", "INFO".white(), w = w),
            LogType::Warn => write!(f, "{:>w$} >", "WARN".yellow(), w = w),
            LogType::Error => write!(f, "{:>w$} >", "ERROR".red(), w = w),
            LogType::Chat => write!(f, "{:>w$} >", "CHAT".cyan(), w = w),
            LogType::Send => write!(f, "{:>w$} >", "SEND".green(), w = w),
            LogType::Console => write!(f, "{:>w$} >", "CONSOLE".magenta(), w = w),
        }
    }
}
