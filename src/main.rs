use futures::{lock::Mutex, prelude::*};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Display, sync::Arc};
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
    let completer = channels_bot.commands.clone();
    cli.set_completer(completer);
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
        bot.lock().await.log(LogType::Info, "Chat handle shut down");
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
        bot.lock()
            .await
            .log(LogType::Info, "Update handle shut down");
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
        bot.lock()
            .await
            .log(LogType::Info, "Console handle shut down");
    });

    client.join(channel_login);

    message_handle.await.unwrap();
    update_handle.await.unwrap();
    console_handle.await.unwrap();

    channels_bot
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
pub type Cli = Arc<linefeed::Interface<linefeed::DefaultTerminal>>;

pub struct BotCommands {
    pub inner: Mutex<Commands<ChannelsBot>>,
}

#[derive(Clone)]
pub struct ChannelsBot {
    cli: Cli,
    commands: Arc<BotCommands>,
}

impl ChannelsBot {
    pub fn new(cli: &Cli, active_bots: ActiveBots) -> Self {
        Self {
            cli: cli.clone(),
            commands: Arc::new(BotCommands {
                inner: Mutex::new(Self::commands()),
            }),
        }
    }

    pub async fn handle_server_message(&mut self, client: &TwitchClient, message: ServerMessage) {
        match message {
            ServerMessage::Join(message) => {
                self.log(LogType::Info, &format!("Joined {}", message.channel_login));
            }
            ServerMessage::Notice(message) => {
                if message.message_text == "Login authentication failed" {
                    panic!("Login authentication failed.");
                }
            }
            ServerMessage::Privmsg(message) => {
                use colored::*;
                let sender_name = match &message.name_color {
                    Some(color) => message
                        .sender
                        .name
                        .truecolor(color.r, color.g, color.b)
                        .to_string(),
                    None => message.sender.name.clone(),
                };
                self.log(
                    LogType::Chat,
                    &format!("{}: {}", sender_name, message.message_text),
                );
                self.perform_commands(
                    client,
                    &message.channel_login,
                    private_to_command_message(&message),
                )
                .await;
            }
            _ => (),
        }
        // for bot in self.active_bots.values_mut() {
        //     bot.handle_server_message(client, &message).await;
        // }
    }

    pub async fn handle_command_message(
        &mut self,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: CommandMessage,
    ) {
        self.perform_commands(client, channel, message).await;

        // for bot in self.active_bots.values_mut() {
        //     bot.handle_command_message(client, channel, message)
        //         .await;
        // }
    }

    pub async fn update(&mut self, client: &TwitchClient, channel: &ChannelLogin, delta_time: f32) {
        // for bot in self.active_bots.values_mut() {
        //     bot.update(client, channel, delta_time).await;
        // }
    }

    pub async fn perform_commands(
        &mut self,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: CommandMessage,
    ) {
        let message_origin = message.sender.origin;
        let commands = self.commands.inner.lock().await;
        let matched = commands.find_commands(&message).collect::<Vec<_>>();
        drop(commands); // Interestingly this line is required to force Rust to drop early
        for (command, args) in matched {
            if let Some(command_reply) = command(self, &message.sender, args) {
                match message_origin {
                    MessageOrigin::Twitch => {
                        self.send_message(client, channel.clone(), command_reply)
                            .await;
                    }
                    MessageOrigin::Console => {
                        self.log(LogType::Console, &command_reply);
                    }
                }
            }
        }
    }

    pub async fn send_message(&self, client: &TwitchClient, channel: String, message: String) {
        self.log(LogType::Send, &format!("{}: {}", channel, message));
        client.say(channel, message).await.unwrap();
    }

    pub fn log(&self, log_type: LogType, message: &str) {
        let mut writer = self.cli.lock_writer_erase().unwrap();
        writeln!(writer, "{} {}", log_type, message).unwrap();
    }

    fn commands() -> Commands<Self> {
        Commands::new(vec![CommandNode::literal(
            ["test"],
            vec![CommandNode::final_node(
                true,
                AuthorityLevel::Viewer as _,
                Arc::new(|_, sender, args| {
                    Some(format!("Got a message from {sender:?}: {args:?}"))
                }),
            )],
        )])
    }
}

impl<Term: linefeed::Terminal> linefeed::Completer<Term> for BotCommands {
    fn complete(
        &self,
        word: &str,
        prompter: &linefeed::Prompter<Term>,
        start: usize,
        end: usize,
    ) -> Option<Vec<linefeed::Completion>> {
        let commands = futures::executor::block_on(self.inner.lock());
        commands.complete(word, prompter, start, end)
    }
}

impl Display for LogType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::*;
        match &self {
            LogType::Info => write!(f, "{:9}", "[INFO]".white()),
            LogType::Warn => write!(f, "{:9}", "[WARN]".yellow()),
            LogType::Error => write!(f, "{:9}", "[ERROR]".red()),
            LogType::Chat => write!(f, "{:9}", "[CHAT]".cyan()),
            LogType::Send => write!(f, "{:9}", "[SEND]".green()),
            LogType::Console => write!(f, "{:9}", "[CONSOLE]".magenta()),
        }
    }
}
