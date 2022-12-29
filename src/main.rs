use futures::{lock::Mutex, prelude::*};
use linefeed::Completer;
use rocket::{get, routes};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Display, sync::Arc};
use tokio_compat_02::FutureExt;
use twitch_irc::{login::StaticLoginCredentials, ClientConfig};

use twitch_bot::prelude::*;

mod bots;
mod main_bot;
mod server;
mod traits;

use main_bot::*;
use traits::*;

pub type BotName = String;
pub type ChannelLogin = String;
pub type ActiveBots = HashSet<BotName>;
pub type Prompter<'a, 'b> = linefeed::Prompter<'a, 'b, linefeed::DefaultTerminal>;
pub type Cli = Arc<linefeed::Interface<linefeed::DefaultTerminal>>;

const CONSOLE_PREFIX_LENGTH: usize = 7;

#[derive(clap::Parser, Debug)]
struct Args {
    #[clap(long)]
    no_cli: bool,
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = <Args as clap::Parser>::parse();

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

    let (main_bot, console_handle, console_abort) = if args.no_cli {
        let main_bot = Arc::new(MutexBot::new(MainBot::new(None, active_bots)));
        (main_bot, None, None)
    } else {
        // Setup CLI
        let cli = Arc::new(linefeed::Interface::new("nertsal-bot").unwrap());
        let main_bot = MainBot::new(Some(&cli), active_bots);
        let main_bot = Arc::new(MutexBot::new(main_bot));
        let completer = main_bot.clone();
        cli.set_completer(completer);

        // Initialize CLI handle
        let bot = Arc::clone(&main_bot);
        let client_clone = client.clone();
        let channel_login_clone = channel_login.clone();
        let (console_handle, console_abort) =
            futures::future::abortable(tokio::spawn(async move {
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
                                    name: "Server".to_owned(),
                                    origin: MessageOrigin::Console,
                                },
                                message_text: input.clone(),
                                authority_level: AuthorityLevel::Server as usize,
                            },
                        )
                        .await;

                    if bot_lock.queue_shutdown {
                        break;
                    }
                }
                bot.lock().await.queue_shutdown = true;
            }));

        (main_bot, Some(console_handle), Some(console_abort))
    };

    // Initialize twitch handle
    let bot = Arc::clone(&main_bot);
    let client_clone = client.clone();
    let (message_handle, message_abort) = futures::future::abortable(tokio::spawn(async move {
        while let Some(message) = incoming_messages.next().await {
            let mut bot_lock = bot.lock().await;
            bot_lock.handle_server_message(&client_clone, message).await;
        }
        let mut bot_lock = bot.lock().await;
        bot_lock.log(LogType::Info, "Chat handle shut down");
        bot_lock.queue_shutdown = true;
    }));

    // Launch server
    let bot = Arc::clone(&main_bot);
    let (server_handle, server_abort) = futures::future::abortable(tokio::spawn(async move {
        use server::*;

        let config = rocket::Config {
            log_level: rocket::log::LogLevel::Off,
            address: std::net::Ipv4Addr::new(127, 0, 0, 1).into(),
            port: 8000,
            ..Default::default()
        };

        {
            let bot_lock = bot.lock().await;
            bot_lock.log(
                LogType::Info,
                &format!("Starting the server on {}:{}", config.address, config.port),
            );
        }

        let result = rocket::custom(config)
            .manage(Arc::clone(&bot))
            .mount("/", routes![index, get_state, events])
            .launch()
            .await;

        let mut bot_lock = bot.lock().await;
        match result {
            Ok(_) => bot_lock.log(LogType::Info, "Server shutdown succesfully"),
            Err(error) => bot_lock.log(
                LogType::Error,
                &format!("Server failed with error: {error}"),
            ),
        }
        bot_lock.queue_shutdown = true;
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
                if let Some(console_abort) = console_abort {
                    console_abort.abort();
                }
                message_abort.abort();
                server_abort.abort();
                break;
            }
        }
        bot.lock().await.queue_shutdown = true;
    });

    client.join(channel_login);
    {
        #![allow(unused_must_use)]
        // Wait for all threads to finish
        update_handle.await;
        server_handle.await;
        message_handle.await;
        if let Some(console_handle) = console_handle {
            console_handle.await;
        }
    }

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

pub fn log(cli: &Option<Cli>, log_type: LogType, message: &str) {
    if let Some(cli) = cli {
        let mut writer = cli.lock_writer_erase().unwrap();
        writeln!(writer, "{} {}", log_type, message).unwrap();
    }
}

pub async fn send_message(
    cli: &Option<Cli>,
    client: &TwitchClient,
    channel: String,
    message: String,
) {
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
