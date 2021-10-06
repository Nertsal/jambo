use super::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum GameJamState {
    Idle,
    Playing,
    Raffle,
}
