use std::fmt::{Debug, Display};

use super::*;

#[async_trait]
pub trait Bot: Send + Sync {
    fn name(&self) -> &str;

    async fn handle_server_message(&mut self, client: &TwitchClient, message: &ServerMessage);

    async fn handle_command_message(&mut self, client: &TwitchClient, message: &CommandMessage);

    async fn update(&mut self, _client: &TwitchClient, _delta_time: f32) {}

    fn update_status(&self, status_text: &str) {
        let path = format!("status/{}.txt", self.name());
        std::fs::write(path, status_text).expect("Could not update bot status");
    }

    fn get_completion_tree(&self) -> Vec<CompletionNode> {
        vec![]
    }
}

pub type CLI = Arc<linefeed::Interface<linefeed::DefaultTerminal>>;

#[async_trait]
pub trait BotLogger {
    async fn send_message(&self, client: &TwitchClient, channel_login: String, message: String) {
        self.log(
            LogType::SendMessage,
            &format!("{}: {}", channel_login, message),
        );
        client.say(channel_login, message).await.unwrap();
    }

    fn log(&self, log_type: LogType, message: &str) {
        let mut writer = self.get_log_cli().lock_writer_erase().unwrap();
        writeln!(writer, "{} {}", log_type, message).unwrap();
    }

    fn get_log_cli(&self) -> &CLI;
}

impl<T> BotLogger for T
where
    T: CommandBot<T> + Sync + Send,
{
    fn get_log_cli(&self) -> &CLI {
        self.get_cli()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LogType {
    Error,
    Info,
    ChatMessage,
    SendMessage,
    ConsoleResponse,
}

impl Display for LogType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::*;
        match &self {
            LogType::Error => write!(f, "{}", "[ERROR]".red()),
            LogType::Info => write!(f, "{}", "[INFO]".yellow()),
            LogType::ChatMessage => write!(f, "{}", "[CHAT]".cyan()),
            LogType::SendMessage => write!(f, "{}", "[SEND]".green()),
            LogType::ConsoleResponse => write!(f, "{}", "[CONSOLE]".magenta()),
        }
    }
}
