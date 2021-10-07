use super::*;

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub current_state: GameJamState,
    pub is_open: bool,
    pub returned_queue: VecDeque<Game>,
    pub games_queue: VecDeque<Game>,
    pub skipped: Vec<Game>,
    pub raffle_viewer_weights: HashMap<String, u32>,
}

impl SaveState {
    pub fn new() -> Self {
        Self {
            current_state: GameJamState::Idle,
            is_open: true,
            returned_queue: VecDeque::new(),
            games_queue: VecDeque::new(),
            skipped: Vec::new(),
            raffle_viewer_weights: HashMap::new(),
        }
    }

    pub fn queue(&self) -> impl Iterator<Item = &Game> {
        self.returned_queue.iter().chain(self.games_queue.iter())
    }
}

pub fn save_into<T: Serialize>(
    value: &T,
    path: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    serde_json::to_writer(file, value)?;
    Ok(())
}

pub fn load_from<T: serde::de::DeserializeOwned>(
    path: impl AsRef<std::path::Path>,
) -> std::io::Result<T> {
    let file = std::io::BufReader::new(std::fs::File::open(path)?);
    Ok(serde_json::from_reader(file)?)
}
