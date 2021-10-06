use bot_core::prelude::*;
use google_sheets4::Sheets;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

mod bot;
mod commands;
mod config;
mod game;
mod google;
mod save_state;
mod state;

use config::*;
use game::*;
use google::*;
use save_state::*;
use state::*;

pub struct GameJamBot {
    // Bot stuff
    cli: CLI,
    config: GameJamConfig,
    commands: Commands<Self, Sender>,

    // Google stuff
    hub: Option<Sheets>,
    update_sheets_queued: bool,

    // Actual data
    played_games: Vec<Game>,
    save_state: SaveState,
    // time_limit: Option<f32>,
}

impl GameJamBot {
    pub fn name() -> &'static str {
        "GameJamBot"
    }

    fn check_message(&mut self, message: &CommandMessage<Sender>) -> Response {
        // Check if waiting for reply
        let state = std::mem::take(&mut self.save_state.current_state);
        match state {
            GameJamState::Waiting { game, .. } => {
                if message.sender.name == game.author {
                    return self.set_current(Some(game));
                }
            }
            state => {
                self.save_state.current_state = state;
            }
        }

        // Try return if auto return is set
        if self.config.auto_return {
            return self.return_game(&message.sender.name);
        }

        None
    }

    fn update(&mut self, delta_time: f32) -> Response {
        match &mut self.save_state.current_state {
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
        save_into(&self.save_state, SAVE_FILE)
    }

    fn load_games(&mut self) -> std::io::Result<()> {
        self.save_state = load_from(SAVE_FILE)?;
        Ok(())
    }
}
