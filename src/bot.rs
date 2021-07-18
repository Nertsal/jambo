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

pub async fn send_message(
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
