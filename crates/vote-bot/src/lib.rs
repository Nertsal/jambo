mod commands;
use std::{collections::HashMap, sync::Arc};

use bot_core::prelude::*;

#[derive(Bot)]
pub struct VoteBot {
    cli: CLI,
    commands: Commands<Self, Sender>,
    vote_mode: VoteMode,
}

impl VoteBot {
    pub fn name() -> &'static str {
        "VoteBot"
    }

    pub fn new(cli: &CLI) -> Box<dyn Bot> {
        Box::new(Self {
            cli: Arc::clone(cli),
            commands: Self::commands(),
            vote_mode: VoteMode::Inactive,
        })
    }

    #[allow(unused_variables)]
    async fn handle_update(
        &mut self,
        client: &TwitchClient,
        channel_login: &String,
        delta_time: f32,
    ) {
    }
}

enum VoteMode {
    Inactive,
    Active { votes: HashMap<String, String> },
}
