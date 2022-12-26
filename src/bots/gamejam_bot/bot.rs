use super::*;

impl BotPerformer for GamejamBot {
    const NAME: &'static str = "GamejamBot";

    fn commands(&self) -> &Commands<Self> {
        &self.commands
    }
}

#[async_trait]
impl Bot for GamejamBot {
    async fn handle_message(
        &mut self,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: &CommandMessage,
    ) {
        if let Some(reply) = self.check_message(message) {
            send_message(&self.cli, client, channel.to_owned(), reply.message).await;
        }
        self.perform(&self.cli.clone(), client, channel, message)
            .await;
    }

    async fn update(&mut self, client: &TwitchClient, channel_login: &String, delta_time: f32) {
        if let Some(reply) = self.update(delta_time) {
            send_message(&self.cli, client, channel_login.clone(), reply.message).await;
        }

        if self.update_sheets_queued {
            if self.config.google_sheet_config.is_some() {
                match self.save_sheets().await {
                    Ok(_) => (),
                    Err(err) => log(
                        &self.cli,
                        LogType::Error,
                        &format!("Error trying to save queue into google sheets: {}", err),
                    ),
                }
            }
            self.update_sheets_queued = false;
        }
    }

    fn complete(
        &self,
        word: &str,
        prompter: &Prompter,
        start: usize,
        end: usize,
    ) -> Option<Vec<linefeed::Completion>> {
        self.commands.complete(word, prompter, start, end)
    }

    fn serialize(&self) -> SerializedBot {
        SerializedBot::Gamejam(Box::new(GamejamSerialized {
            config: self.config.clone(),
            state: self.state.clone(),
        }))
    }
}
