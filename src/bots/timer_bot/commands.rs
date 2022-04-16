use std::sync::Arc;

use super::*;

impl TimerBot {
    fn timer_from_str(&mut self, mode: &str, time: Option<&str>) -> Response {
        let time = match time {
            Some(time) => match Timer::parse_duration(time) {
                Ok(time) => Some(time),
                Err(err) => return Some(format!("Failed to parse time argument: {err}")),
            },
            None => None,
        };

        match mode {
            "set" => self.timer_set(TimerMode::Idle, time),
            "countup" => self.timer_set(TimerMode::Countup, time),
            "countdown" => self.timer_set(TimerMode::Countdown, time),
            _ => return None,
        }
    }

    fn timer_pause(&mut self, paused: bool) -> Response {
        self.timer.paused = paused;
        match paused {
            true => Some(format!("Timer has been paused")),
            false => Some(format!("Timer has been resumed")),
        }
    }

    fn timer_set(&mut self, mode: TimerMode, time: Option<Time>) -> Response {
        if let Some(time) = time {
            self.timer.time = time;
        }
        self.timer.paused = match mode {
            TimerMode::Idle => true,
            TimerMode::Countdown => false,
            TimerMode::Countup => false,
        };
        self.timer.mode = mode;

        Some(format!(
            "Changed timer to {} from {}",
            self.timer.mode,
            Timer::format_duration(self.timer.time)
        ))
    }

    pub fn commands() -> Commands<Self> {
        let pause = CommandBuilder::<Self, _>::new()
            .choice(["pause", "continue"])
            .finalize(
                true,
                AuthorityLevel::Broadcaster as _,
                Arc::new(|bot, _, args| {
                    let paused = match args[0].as_str() {
                        "pause" => true,
                        "continue" => false,
                        _ => return None,
                    };
                    bot.timer_pause(paused)
                }),
            );

        let set_time = CommandBuilder::<Self, _>::new().word().finalize(
            true,
            AuthorityLevel::Moderator as _,
            Arc::new(|bot, _, args| bot.timer_from_str(&args[0], Some(&args[1]))),
        );

        let set_no_time = CommandBuilder::<Self, _>::new().finalize(
            true,
            AuthorityLevel::Moderator as _,
            Arc::new(|bot, _, args| bot.timer_from_str(&args[0], None)),
        );

        let set = CommandBuilder::new()
            .choice(["set", "countup", "countdown"])
            .split([set_time, set_no_time]);

        Commands {
            commands: vec![CommandBuilder::new()
                .literal(["!timer"])
                .split([pause, set])],
        }
    }
}
