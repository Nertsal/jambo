use std::sync::Arc;

use super::*;

impl TimerBot {
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
        Commands {
            commands: vec![CommandNode::literal(
                ["!timer"],
                vec![
                    CommandNode::argument_choice(
                        ["pause", "continue"],
                        vec![CommandNode::final_node(
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
                        )],
                    ),
                    CommandNode::argument_choice(
                        ["set", "countup", "countdown"],
                        vec![
                            CommandNode::argument(
                                ArgumentType::Word,
                                vec![CommandNode::final_node(
                                    true,
                                    AuthorityLevel::Moderator as _,
                                    Arc::new(|bot, _, args| {
                                        match Timer::parse_duration(args[1].as_str()) {
                                            Ok(time) => match args[0].as_str() {
                                                "set" => bot.timer_set(TimerMode::Idle, Some(time)),
                                                "countup" => {
                                                    bot.timer_set(TimerMode::Countup, Some(time))
                                                }
                                                "countdown" => {
                                                    bot.timer_set(TimerMode::Countdown, Some(time))
                                                }
                                                _ => return None,
                                            },
                                            Err(err) => Some(format!(
                                                "Failed to parse time argument: {err}"
                                            )),
                                        }
                                    }),
                                )],
                            ),
                            CommandNode::final_node(
                                true,
                                AuthorityLevel::Moderator as _,
                                Arc::new(|bot, _, args| match args[0].as_str() {
                                    "set" => bot.timer_set(TimerMode::Idle, None),
                                    "countup" => bot.timer_set(TimerMode::Countup, None),
                                    "countdown" => bot.timer_set(TimerMode::Countdown, None),
                                    _ => return None,
                                }),
                            ),
                        ],
                    ),
                ],
            )],
        }
    }
}
