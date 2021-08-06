use std::sync::Arc;

use bot_core::prelude::*;

mod commands;
mod timer;

use timer::*;

#[derive(Bot)]
pub struct TimerBot {
    channel_login: String,
    cli: CLI,
    commands: Commands<Self, Sender>,
    timer: Timer,
}

impl TimerBot {
    pub fn name() -> &'static str {
        "TimerBot"
    }

    pub fn new(cli: &CLI, channel_login: &str) -> Box<dyn Bot> {
        Box::new(Self {
            channel_login: channel_login.to_owned(),
            cli: Arc::clone(cli),
            commands: Self::commands(),
            timer: Timer::from_status().unwrap_or_default(),
        })
    }

    async fn handle_update(&mut self, _client: &TwitchClient, delta_time: f32) {
        self.update_timer(delta_time);
    }

    fn update_timer(&mut self, delta_time: f32) {
        self.timer.update(delta_time);
        self.update_status(&self.timer.time_status());
    }
}
