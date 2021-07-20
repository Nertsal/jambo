use std::sync::Arc;

use super::*;

impl CommandBot<Self> for TimerBot {
    fn get_commands(&self) -> &BotCommands<Self> {
        &self.commands
    }

    fn get_cli(&self) -> &CLI {
        &self.cli
    }
}

impl TimerBot {
    fn set_timer(&mut self, timer: Timer) {
        self.timer = timer;
    }

    fn timer_pause(&mut self, pause: bool) {
        self.timer.paused = pause;
    }

    pub fn commands() -> BotCommands<Self> {
        BotCommands {
            commands: vec![CommandNode::Literal {
                literals: vec!["!timer".to_owned()],
                child_nodes: vec![
                    CommandNode::Literal {
                        literals: vec!["pause".to_owned()],
                        child_nodes: vec![CommandNode::Final {
                            authority_level: AuthorityLevel::Broadcaster,
                            command: Arc::new(|bot, _, _| {
                                bot.timer_pause(true);
                                Some(format!("Timer has been paused"))
                            }),
                        }],
                    },
                    CommandNode::Literal {
                        literals: vec!["continue".to_owned()],
                        child_nodes: vec![CommandNode::Final {
                            authority_level: AuthorityLevel::Broadcaster,
                            command: Arc::new(|bot, _, _| {
                                bot.timer_pause(false);
                                Some(match bot.timer.timer_mode {
                                    TimerMode::Idle => format!("Timer is in idle state. Call !timer countdown or !timer countup to start the timer."),
                                    _ => format!("Timer has been unpaused"),
                                })
                            }),
                        }],
                    },
                    CommandNode::ArgumentChoice {
                        choices: vec![
                            "set".to_owned(),
                            "countup".to_owned(),
                            "countdown".to_owned(),
                        ],
                        child_nodes: vec![
                            CommandNode::Final {
                                authority_level: AuthorityLevel::Broadcaster,
                                command: Arc::new(|bot, _, mut args| {
                                    let mode = args.remove(0);
                                    Timer::new_str(bot.timer.time, &mode).map_or(None, |timer| {
                                        if !timer.paused {
                                            let reply = format!("Timer's mode has been updated");
                                            bot.set_timer(timer);
                                            Some(reply)
                                        } else {
                                            None
                                        }
                                    })
                                }),
                            },
                            CommandNode::Argument {
                                argument_type: ArgumentType::Line,
                                child_nodes: vec![CommandNode::Final {
                                    authority_level: AuthorityLevel::Broadcaster,
                                    command: Arc::new(|bot, _, mut args| {
                                        let mode = args.remove(0);
                                        match Timer::parse_duration(args.remove(0).as_ref()) {
                                            Ok(time) => {
                                                Timer::new_str(time, &mode).map_or(None, |timer| {
                                                    let reply = format!("Timer has been set");
                                                    bot.set_timer(timer);
                                                    Some(reply)
                                                })
                                            }
                                            Err(_) => {
                                                Some(format!("Could not parse time argument"))
                                            }
                                        }
                                    }),
                                }],
                            },
                        ],
                    },
                ],
            }],
        }
    }
}
