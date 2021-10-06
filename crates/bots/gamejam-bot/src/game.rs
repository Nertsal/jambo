use super::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Game {
    pub author: String,
    pub link: String,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum GameType {
    Queued,
    Current,
    Skipped,
    Played,
}
