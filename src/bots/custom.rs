use super::*;

pub struct CustomBot {
    cli: Cli,
    commands: Commands<Self>,
}

impl BotPerformer for CustomBot {
    fn commands(&self) -> &Commands<Self> {
        &self.commands
    }
}

#[async_trait]
impl Bot for CustomBot {
    async fn handle_message(
        &mut self,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: &CommandMessage,
    ) {
        self.perform(&self.cli.clone(), client, channel, message)
            .await;
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
}

impl CustomBot {
    pub const NAME: &'static str = "CustomBot";

    pub fn subbot(cli: &Cli) -> Box<dyn Bot> {
        Box::new(Self {
            cli: cli.clone(),
            commands: Commands::new(vec![]),
        })
    }
}
