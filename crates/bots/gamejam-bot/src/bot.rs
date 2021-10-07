use super::*;

#[async_trait]
impl Bot for GameJamBot {
    fn name(&self) -> &str {
        Self::name()
    }

    async fn handle_server_message(&mut self, client: &TwitchClient, message: &ServerMessage) {
        match message {
            ServerMessage::Privmsg(private_message) => {
                let message = private_to_command_message(private_message);
                if let Some(reply) = self.check_message(&message) {
                    self.send_message(client, private_message.channel_login.clone(), reply)
                        .await;
                }
                perform_commands(
                    self,
                    client,
                    private_message.channel_login.clone(),
                    &message,
                )
                .await;
            }
            _ => (),
        }
    }

    async fn update(&mut self, client: &TwitchClient, channel_login: &String, delta_time: f32) {
        if let Some(reply) = self.update(delta_time) {
            self.send_message(client, channel_login.clone(), reply)
                .await;
        }

        if self.update_sheets_queued {
            if self.config.google_sheet_config.is_some() {
                match self.save_sheets().await {
                    Ok(_) => (),
                    Err(err) => self.log(
                        LogType::Error,
                        &format!("Error trying to save queue into google sheets: {}", err),
                    ),
                }
            }
            self.update_sheets_queued = false;
        }
    }

    async fn handle_command_message(
        &mut self,
        client: &TwitchClient,
        channel_login: &String,
        message: &CommandMessage<Sender>,
    ) {
        perform_commands(self, client, channel_login.clone(), &message).await;
    }

    fn get_completion_tree(&self) -> Vec<CompletionNode> {
        commands_to_completion(&self.get_commands().commands)
    }
}
