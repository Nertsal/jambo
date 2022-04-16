use std::collections::HashMap;

use super::*;

mod commands;

pub struct VoteBot {
    cli: Cli,
    commands: Commands<Self>,
    vote_mode: VoteMode,
}

enum VoteMode {
    Inactive,
    Active { votes: HashMap<String, String> },
}

impl VoteBot {
    pub fn new(cli: &Cli) -> Box<dyn Bot> {
        Box::new(Self {
            cli: Arc::clone(cli),
            commands: Self::commands(),
            vote_mode: VoteMode::Inactive,
        })
    }
}

impl BotPerformer for VoteBot {
    const NAME: &'static str = "VoteBot";

    fn commands(&self) -> &Commands<Self> {
        &self.commands
    }
}

#[async_trait]
impl Bot for VoteBot {
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
