use google_sheets4::{hyper, hyper_rustls, Sheets};
use std::collections::{HashMap, VecDeque};
use yup_oauth2 as oauth2;

use super::*;

mod bot;
mod bot_state;
mod commands;
mod config;
mod google;

use bot_state::*;
use config::*;
use google::*;

pub struct GamejamBot {
    // Bot stuff
    cli: Cli,
    config: GamejamConfig,
    commands: Commands<Self>,

    // Google stuff
    hub: Option<Sheets>,
    update_sheets_queued: bool,

    // Actual data
    state: GamejamState,
}

#[derive(Serialize)]
pub struct GamejamSerialized {
    config: GamejamConfig,
    state: GamejamState,
}

impl GamejamBot {
    pub fn name() -> &'static str {
        "GamejamBot"
    }

    fn check_message(&mut self, message: &CommandMessage) -> Response {
        // Check if waiting for reply
        let state = std::mem::take(&mut self.state.current_state);
        match state {
            GameJamState::Waiting { game, .. } => {
                if game.authors.contains(&message.sender.name) {
                    return self.set_current(Some(game));
                }
            }
            state => {
                self.state.current_state = state;
            }
        }

        // Try return if auto return is set
        if self.config.auto_return {
            return self.return_game(&message.sender.name);
        }

        None
    }

    fn update(&mut self, delta_time: f32) -> Response {
        match &mut self.state.current_state {
            GameJamState::Waiting { time_limit, .. } => {
                *time_limit -= delta_time;
                if *time_limit <= 0.0 {
                    self.skip(true)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn save_games(&mut self) -> std::io::Result<()> {
        self.update_sheets_queued = true;
        save_into(&self.state, SAVE_FILE)
    }
}
