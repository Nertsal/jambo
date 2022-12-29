use super::*;

#[async_trait]
pub trait Bot: Send {
    async fn handle_message(
        &mut self,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: &CommandMessage,
    );

    async fn update(&mut self, client: &TwitchClient, channel: &ChannelLogin, delta_time: f32) {
        #![allow(unused_variables)]
    }

    fn complete(
        &self,
        word: &str,
        prompter: &Prompter,
        start: usize,
        end: usize,
    ) -> Option<Vec<linefeed::Completion>>;

    fn serialize(&self) -> SerializedBot;
}

#[async_trait]
pub trait BotPerformer {
    const NAME: &'static str;

    fn commands(&self) -> &Commands<Self>;

    async fn perform(
        &mut self,
        cli: &Option<Cli>,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: &CommandMessage,
    ) {
        let message_origin = &message.sender.origin;
        let commands = self.commands();
        let matched = commands.find_commands(message).collect::<Vec<_>>();
        for (command, args) in matched {
            if let Some(mut response) = command(self, &message.sender, args) {
                if let MessageOrigin::Twitch = message_origin {
                    response.send_to_twitch = true;
                }
                if response.send_to_twitch {
                    send_message(cli, client, channel.clone(), response.message).await;
                } else {
                    log(cli, LogType::Console, &response.message);
                }
            }
        }
    }

    /// Write bot's status into a status file
    fn update_status(&self, status_text: &str) {
        let path = format!("status/{}.txt", Self::NAME);
        std::fs::write(path, status_text).expect("Could not update bot status");
    }
}
