use super::*;
use async_trait::async_trait;
use std::fmt::Display;

#[async_trait]
pub trait BotLogger<U, S> {
    fn get_log_cli(&self) -> &CLI;

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
}

impl<T: CommandBot<U, S>, U: Sync + Send, S> BotLogger<U, S> for T {
    fn get_log_cli(&self) -> &CLI {
        self.get_cli()
    }
}

impl<U: Sync + Send, S> BotLogger<U, S> for dyn CommandBot<U, S> {
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
