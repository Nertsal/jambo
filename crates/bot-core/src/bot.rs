use async_trait::async_trait;
use bot_commands::{Sender, TwitchClient};
use bot_completion::CompletionNode;
use nertsal_commands::CommandMessage;
use twitch_irc::message::ServerMessage;

#[async_trait]
pub trait Bot: Send + Sync {
    fn name(&self) -> &str;

    async fn handle_server_message(&mut self, client: &TwitchClient, message: &ServerMessage);

    async fn handle_command_message(
        &mut self,
        client: &TwitchClient,
        message: &CommandMessage<Sender>,
    );

    async fn update(&mut self, _client: &TwitchClient, _delta_time: f32) {}

    fn update_status(&self, status_text: &str) {
        let path = format!("status/{}.txt", self.name());
        std::fs::write(path, status_text).expect("Could not update bot status");
    }

    fn get_completion_tree(&self) -> Vec<CompletionNode> {
        vec![]
    }
}
