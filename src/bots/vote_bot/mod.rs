use std::collections::HashMap;

use super::*;

mod commands;

pub struct VoteBot {
    cli: Option<Cli>,
    commands: Commands<Self>,
    state: VoteState,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoteState {
    vote_mode: VoteMode,
    last_vote: Vec<(String, usize)>,
}

#[derive(Debug, Serialize)]
pub struct VoteSerialized {
    state: VoteState,
}

#[derive(Clone, Debug, Serialize)]
enum VoteMode {
    Inactive,
    Active { votes: HashMap<String, String> },
}

impl VoteBot {
    pub fn new_boxed(cli: &Option<Cli>) -> Box<dyn Bot> {
        Box::new(Self {
            cli: cli.clone(),
            commands: Self::commands(),
            state: VoteState {
                vote_mode: VoteMode::Inactive,
                last_vote: Vec::new(),
            },
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

    fn serialize(&self) -> SerializedBot {
        SerializedBot::Vote(VoteSerialized {
            state: self.state.clone(),
        })
    }
}
