use super::*;

const CONFIG_FILE: &str = "config/gamejam/gamejam_config.json";
pub const SAVE_FILE: &str = "config/gamejam/gamejam_nertsalbot.json";
pub const PLAYED_GAMES_FILE: &str = "config/gamejam/games_played.json";

macro_rules! load {
    ( $path: expr ) => {
        match load_from($path) {
            Ok(value) => value,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => Default::default(),
                _ => panic!("Error loading {}: {}", $path, err),
            },
        }
    };
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum ReturnMode {
    Back,
    Front,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GamejamConfig {
    pub multiple_submissions: bool,
    pub queue_mode: bool,
    pub return_mode: ReturnMode,
    pub auto_return: bool,
    pub response_time_limit: Option<u64>,
    pub link_start: Option<String>,
    pub allow_direct_link_submit: bool,
    pub allow_multiple_authors_submit: bool,
    pub raffle_default_weight: u32,
    pub google_sheet_config: Option<GoogleSheetConfig>,
}

impl GamejamBot {
    pub fn new(cli: &Option<Cli>) -> Box<dyn Bot> {
        // Read config
        let config: GamejamConfig = serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open(CONFIG_FILE).unwrap(),
        ))
        .unwrap();

        // Load bot state
        let mut state: GamejamState = load!(SAVE_FILE);

        // Load played games
        state.submissions.played_games = load!(PLAYED_GAMES_FILE);

        // Initialize bot
        let mut bot = Self {
            cli: cli.clone(),
            config,
            commands: Self::commands(),
            hub: None,
            update_sheets_queued: true,
            state,
        };

        // Initialize google sheets
        if bot.config.google_sheet_config.is_some() {
            let service_key: oauth2::ServiceAccountKey = serde_json::from_reader(
                std::io::BufReader::new(std::fs::File::open("secrets/service_key.json").unwrap()),
            )
            .expect("Failed to read google service key");
            let auth = futures::executor::block_on(
                oauth2::ServiceAccountAuthenticator::builder(service_key).build(),
            )
            .expect("Failed to create an authenticator for google service");

            bot.hub = Some(Sheets::new(
                hyper::Client::builder().build(
                    hyper_rustls::HttpsConnectorBuilder::new()
                        .with_native_roots()
                        .https_only()
                        .enable_http1()
                        .build(),
                ),
                auth,
            ));
        }

        Box::new(bot)
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
