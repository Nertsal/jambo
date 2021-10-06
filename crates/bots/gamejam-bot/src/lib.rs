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
        // if let Some(_) = self.time_limit {
        //     let game = self.games_state.current_game.as_ref().unwrap();
        //     if message.sender.name == game.author {
        //         self.time_limit = None;
        //         return Some(format!("Now playing {}. ", game));
        //     }
        // }
        // if self.config.auto_return {
        //     return self.return_game(&message.sender.name);
        // }
        None
    }

    fn update(&mut self, delta_time: f32) -> Response {
        // if let Some(time) = &mut self.time_limit {
        //     *time -= delta_time;
        //     if *time <= 0.0 {
        //         return self.skip(true);
        //     }
        // }
        None
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
