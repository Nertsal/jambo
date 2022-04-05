use super::*;

mod queue;
mod submissions;

pub use queue::*;
pub use submissions::*;

pub type Luck = u32;

#[derive(Default, Serialize, Deserialize)]
pub struct BotState {
    pub current_state: GameJamState,
    #[serde(flatten)]
    pub submissions: Submissions,
    pub is_queue_open: bool,
    pub raffle_weights: HashMap<String, Luck>,
}

#[derive(Serialize, Deserialize)]
pub enum GameJamState {
    Idle,
    Waiting { time_limit: f32, game: Submission },
    Playing { game: Submission },
    Raffle { joined: HashMap<String, Luck> },
}

impl GameJamState {
    pub fn current(&self) -> Option<&Submission> {
        match self {
            Self::Playing { game } | Self::Waiting { game, .. } => Some(game),
            _ => None,
        }
    }

    pub fn current_mut(&mut self) -> Option<&mut Submission> {
        match self {
            Self::Playing { game } | Self::Waiting { game, .. } => Some(game),
            _ => None,
        }
    }
}

impl Default for GameJamState {
    fn default() -> Self {
        Self::Idle
    }
}
