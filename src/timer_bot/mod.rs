use super::*;

mod commands;
mod timer;

use timer::*;

pub struct TimerBot {
    channel_login: String,
    commands: BotCommands<Self>,
    timer: Timer,
}

impl TimerBot {
    pub fn name() -> &'static str {
        "TimerBot"
    }

    pub fn new(channel_login: &str) -> Box<dyn Bot> {
        Box::new(Self {
            channel_login: channel_login.to_owned(),
            commands: Self::commands(),
            timer: Timer::from_status().unwrap_or_default(),
        })
    }

    fn update_timer(&mut self, delta_time: f32) {
        self.timer.update(delta_time);
        self.update_status(&self.timer.time_status());
    }
}

#[async_trait]
impl Bot for TimerBot {
    fn name(&self) -> &str {
        Self::name()
    }

    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &ServerMessage,
    ) {
        match message {
            ServerMessage::Privmsg(message) => {
                check_command(
                    self,
                    client,
                    self.channel_login.clone(),
                    &CommandMessage::from(message),
                )
                .await;
            }
            _ => (),
        };
    }

    async fn update(
        &mut self,
        _client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        delta_time: f32,
    ) {
        self.update_timer(delta_time);
    }
}
