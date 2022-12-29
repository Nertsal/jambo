use std::collections::HashMap;

use super::*;

use crate::bots::*;

mod bot;
mod bots;
mod commands;
mod mutex;

pub use bot::*;
use bots::*;
pub use mutex::*;

type BotConstructor = fn(&Option<Cli>) -> Box<dyn Bot>;

// -- Modify this section to include a new bot into the main bot --

#[derive(Serialize)]
pub enum SerializedBot {
    // Insert here
    Custom(CustomSerialized),
    Quote(QuoteSerialized),
    Timer(TimerSerialized),
    Vote(VoteSerialized),
    Gamejam(Box<GamejamSerialized>),
}

fn constructors() -> impl IntoIterator<Item = (BotName, BotConstructor)> {
    // Add a line below to make constructing the bot possible
    [
        // Insert here
        (CustomBot::NAME.to_owned(), CustomBot::new_boxed as _),
        (QuoteBot::NAME.to_owned(), QuoteBot::new_boxed as _),
        (TimerBot::NAME.to_owned(), TimerBot::new_boxed as _),
        (VoteBot::NAME.to_owned(), VoteBot::new_boxed as _),
        (GamejamBot::NAME.to_owned(), GamejamBot::new_boxed as _),
    ]
}

// -- End of the section, do not modify anything below --
