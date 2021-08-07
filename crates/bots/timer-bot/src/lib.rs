use std::sync::Arc;

use bot_core::prelude::*;

mod commands;
mod timer;

use timer::*;

#[derive(Bot)]
pub struct TimerBot {
    cli: CLI,
    commands: Commands<Self, Sender>,
    timer: Timer,
}

impl TimerBot {
    pub fn name() -> &'static str {
        "TimerBot"
    }

    pub fn new(cli: &CLI) -> Box<dyn Bot> {
        Box::new(Self {
            cli: Arc::clone(cli),
            commands: Self::commands(),
            timer: Timer::from_status().unwrap_or_default(),
        })
    }

    #[allow(unused_variables)]
    async fn handle_update(
        &mut self,
        client: &TwitchClient,
        channel_login: &String,
        delta_time: f32,
    ) {
        self.update_timer(delta_time);
    }

    fn update_timer(&mut self, delta_time: f32) {
        self.timer.update(delta_time);
        self.update_status(&self.timer.time_status());
    }
}
