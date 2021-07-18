use std::fmt::Debug;

use super::*;

#[async_trait]
pub trait Bot: Send + Sync {
    fn name(&self) -> &str;

    async fn handle_server_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &ServerMessage,
    );

    async fn handle_command_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &CommandMessage,
    );

    async fn update(
        &mut self,
        _client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        _delta_time: f32,
    ) {
    }

    fn update_status(&self, status_text: &str) {
        let path = format!("status/{}.txt", self.name());
        std::fs::write(path, status_text).expect("Could not update bot status");
    }
}

#[async_trait]
pub trait BotLogger {
    async fn send_message(
        &self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        channel_login: String,
        message: String,
    ) {
        self.log(
            LogType::SendMessage,
            &format!("{}: {}", channel_login, message),
        );
        client.say(channel_login, message).await.unwrap();
    }

    fn log(&self, log_type: LogType, message: &str) {
        log_type.log(message);
    }
}

impl<T> BotLogger for T where T: CommandBot<T> + Sync + Send {}

#[derive(Debug, Clone, Copy)]
pub enum LogType {
    Error,
    Info,
    ChatMessage,
    SendMessage,
}

impl LogType {
    fn log(&self, message: &str) {
        match &self {
            LogType::Error => bunt::print!("{$red}[ERROR]{/$}"),
            LogType::Info => bunt::print!("{$yellow}[INFO]{/$}"),
            LogType::ChatMessage => bunt::print!("{$cyan}[CHAT]{/$}"),
            LogType::SendMessage => bunt::print!("{$green}[SEND]{/$}"),
        }
        bunt::println!(" {}", message);
    }
}
