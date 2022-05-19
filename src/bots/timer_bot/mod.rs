use super::*;

mod commands;
mod timer;

use timer::*;

pub struct TimerBot {
    cli: Option<Cli>,
    commands: Commands<Self>,
    timer: Timer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimerSerialized {
    state: Timer,
}

impl TimerBot {
    pub fn new(cli: &Option<Cli>) -> Box<dyn Bot> {
        Box::new(Self {
            cli: cli.clone(),
            commands: Self::commands(),
            timer: Timer::from_status().unwrap_or_default(),
        })
    }

    fn update_timer(&mut self, delta_time: f32) {
        self.timer.update(delta_time);
        self.update_status(&self.timer.time_status());
    }
}

impl BotPerformer for TimerBot {
    const NAME: &'static str = "TimerBot";

    fn commands(&self) -> &Commands<Self> {
        &self.commands
    }
}

#[async_trait]
impl Bot for TimerBot {
    async fn handle_message(
        &mut self,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: &CommandMessage,
    ) {
        self.perform(&self.cli.clone(), client, channel, message)
            .await;
    }

    async fn update(&mut self, _client: &TwitchClient, _channel: &String, delta_time: f32) {
        self.update_timer(delta_time);
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
        SerializedBot::Timer(TimerSerialized {
            state: self.timer.clone(),
        })
    }
}
