use super::*;

mod commands;

struct Timer {
    time: std::time::Duration,
    paused: bool,
    timer_mode: TimerMode,
}

#[derive(Clone, Copy)]
enum TimerMode {
    Idle,
    Countdown,
    Countup,
}

impl Timer {
    fn from_status() -> Result<Self, Box<dyn std::error::Error>> {
        let time = humantime::parse_duration(&std::fs::read_to_string(format!(
            "status/{}.txt",
            TimerBot::name()
        ))?)?;
        Ok(Self {
            time,
            paused: true,
            timer_mode: TimerMode::Idle,
        })
    }

    fn new_str(time: std::time::Duration, mode: &str) -> Result<Self, ()> {
        let (paused, timer_mode) = match mode {
            "set" => (true, TimerMode::Idle),
            "countdown" => (false, TimerMode::Countdown),
            "countup" => (false, TimerMode::Countup),
            _ => {
                return Err(());
            }
        };
        Ok(Self {
            time,
            paused,
            timer_mode,
        })
    }

    fn update(&mut self, delta_time: f32) {
        if !self.paused {
            let delta = std::time::Duration::from_secs_f32(delta_time);
            match self.timer_mode {
                TimerMode::Idle => (),
                TimerMode::Countdown => {
                    self.time = self.time.checked_sub(delta).unwrap_or_default();
                }
                TimerMode::Countup => {
                    self.time = self.time.checked_add(delta).unwrap_or_default();
                }
            }
        }
    }

    fn time_status(&self) -> String {
        humantime::format_duration(self.time).to_string()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            paused: true,
            time: std::time::Duration::from_secs(0),
            timer_mode: TimerMode::Idle,
        }
    }
}

pub struct TimerBot {
    channel_login: String,
    commands: BotCommands<Self>,
    timer: Timer,
}

impl TimerBot {
    pub fn name() -> &'static str {
        "TimerBot"
    }

    pub fn new(channel_login: &String) -> Self {
        Self {
            channel_login: channel_login.clone(),
            commands: Self::commands(),
            timer: Timer::from_status().unwrap_or_default(),
        }
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
                check_command(self, client, self.channel_login.clone(), message).await;
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
