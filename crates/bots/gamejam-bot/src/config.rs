use super::*;

pub const CONFIG_FILE: &'static str = "config/gamejam/gamejam_config.json";
pub const SAVE_FILE: &'static str = "config/gamejam/gamejam_nertsalbot.json";
pub const PLAYED_GAMES_FILE: &'static str = "config/gamejam/games_played.json";

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum ReturnMode {
    Back,
    Front,
}

#[derive(Serialize, Deserialize)]
pub struct GameJamConfig {
    pub multiple_submissions: bool,
    pub queue_mode: bool,
    pub return_mode: ReturnMode,
    pub auto_return: bool,
    pub response_time_limit: Option<u64>,
    pub link_start: Option<String>,
    pub allow_direct_link_submit: bool,
    pub raffle_default_weight: usize,
    pub google_sheet_config: Option<GoogleSheetConfig>,
}

impl GameJamBot {
    pub fn new(cli: &CLI) -> Box<dyn Bot> {
        let config: GameJamConfig = serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open(CONFIG_FILE).unwrap(),
        ))
        .unwrap();

        let mut bot = Self {
            cli: Arc::clone(cli),
            config,
            commands: Self::commands(),
            played_games: Vec::new(),
            save_state: SaveState::new(),
            hub: None,
            update_sheets_queued: true,
        };

        if bot.config.google_sheet_config.is_some() {
            let service_key: yup_oauth2::ServiceAccountKey = serde_json::from_reader(
                std::io::BufReader::new(std::fs::File::open("secrets/service_key.json").unwrap()),
            )
            .unwrap();
            let auth = async_std::task::block_on(
                yup_oauth2::ServiceAccountAuthenticator::builder(service_key).build(),
            )
            .unwrap();

            bot.hub = Some(Sheets::new(
                hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
                auth,
            ));
        }

        bot.log(
            LogType::Info,
            &format!("Loading GameJamBot data from {}", SAVE_FILE),
        );
        match bot.load_games() {
            Ok(_) => bot.log(
                LogType::Info,
                &format!("Successfully loaded GameJamBot data"),
            ),
            Err(error) => {
                use std::io::ErrorKind;
                match error.kind() {
                    ErrorKind::NotFound => {
                        ("Using default GameJamBot data");
                        bot.save_games().unwrap();
                    }
                    _ => panic!("Error loading GameJamBot data: {}", error),
                }
            }
        }
        match load_from(PLAYED_GAMES_FILE) {
            Ok(played_games) => bot.played_games = played_games,
            Err(error) => {
                use std::io::ErrorKind;
                match error.kind() {
                    ErrorKind::NotFound => {
                        save_into(&bot.played_games, PLAYED_GAMES_FILE).unwrap();
                    }
                    _ => panic!("Error loading GameJamBot data: {}", error),
                }
            }
        }
        Box::new(bot)
    }
}
