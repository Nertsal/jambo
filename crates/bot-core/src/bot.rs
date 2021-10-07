use async_trait::async_trait;
use bot_commands::{Sender, TwitchClient};
use bot_completion::CompletionNode;
use nertsal_commands::CommandMessage;
use twitch_irc::message::ServerMessage;

/// Trait representing a bot.
#[async_trait]
pub trait Bot: Send + Sync {
    /// Bot's name
    fn name(&self) -> &str;

    /// Handle a message received from the twitch server
    async fn handle_server_message(&mut self, client: &TwitchClient, message: &ServerMessage);

    /// Handle a message received from console
    async fn handle_command_message(
        &mut self,
        client: &TwitchClient,
        channel_login: &String,
        message: &CommandMessage<Sender>,
    );

    /// Update the bot
    #[allow(unused_variables)]
    async fn update(&mut self, client: &TwitchClient, channel_login: &String, delta_time: f32) {}

    /// Returns a command completion tree for auto-completion in the console
    fn get_completion_tree(&self) -> Vec<CompletionNode> {
        vec![]
    }

    /// Write bot's status into a status file
    fn update_status(&self, status_text: &str) {
        let path = format!("status/{}.txt", self.name());
        std::fs::write(path, status_text).expect("Could not update bot status");
    }
}
