mod commands;
use std::{collections::HashMap, sync::Arc};

use bot_core::prelude::*;

#[derive(Bot)]
pub struct VoteBot {
    channel_login: String,
    cli: CLI,
    commands: Commands<Self, Sender>,
    vote_mode: VoteMode,
}

impl VoteBot {
    pub fn name() -> &'static str {
        "VoteBot"
    }

    pub fn new(cli: &CLI, channel_login: &str) -> Box<dyn Bot> {
        Box::new(Self {
            channel_login: channel_login.to_owned(),
            cli: Arc::clone(cli),
            commands: Self::commands(),
            vote_mode: VoteMode::Inactive,
        })
    }

    async fn handle_update(&mut self, _client: &TwitchClient, _delta_time: f32) {}
}

enum VoteMode {
    Inactive,
    Active { votes: HashMap<String, String> },
}
