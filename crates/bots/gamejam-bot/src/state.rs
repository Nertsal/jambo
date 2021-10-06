use super::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum GameJamState {
    Idle,
    Waiting { time_limit: f32 },
    Playing,
    Raffle { joined: HashMap<String, u32> },
}
