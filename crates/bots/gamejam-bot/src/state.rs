use super::*;

#[derive(Serialize, Deserialize)]
pub enum GameJamState {
    Idle,
    Waiting { time_limit: f32, game: Game },
    Playing { game: Game },
    Raffle { joined: HashMap<String, u32> },
}

impl Default for GameJamState {
    fn default() -> Self {
        Self::Idle
    }
}
