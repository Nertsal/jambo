use super::*;
use google_sheets4::Sheets;
use std::collections::VecDeque;

mod commands;

#[derive(Serialize, Deserialize)]
pub struct GameJamConfig {
    queue_mode: bool,
    response_time_limit: Option<u64>,
    link_start: Option<String>,
    allow_direct_link_submit: bool,
    raffle_default_weight: usize,
    google_sheet_config: Option<GoogleSheetConfig>,
}

#[derive(Serialize, Deserialize)]
struct GoogleSheetConfig {
    sheet_id: String,
    cell_format: GoogleSheetCellFormat,
}

#[derive(Serialize, Deserialize)]
struct GoogleSheetCellFormat {
    color_queued: Option<google_sheets4::api::Color>,
    color_current: Option<google_sheets4::api::Color>,
    color_skipped: Option<google_sheets4::api::Color>,
    color_played: Option<google_sheets4::api::Color>,
}

pub struct GameJamBot {
    channel_login: String,
    config: GameJamConfig,
    commands: BotCommands<Self>,
    save_file: String,
    played_games_file: String,
    played_games: Vec<Game>,
    games_state: GamesState,
    time_limit: Option<Instant>,
    hub: Option<Sheets>,
    update_sheets: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct Game {
    author: String,
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
enum GameType {
    Queued,
    Current,
    Skipped,
    Played,
}

#[derive(Serialize, Deserialize)]
struct GamesState {
    is_open: bool,
    current_game: Option<Game>,
    returned_queue: VecDeque<Game>,
    games_queue: VecDeque<Game>,
    skipped: Vec<Game>,
    raffle: Raffle,
}

impl GamesState {
    fn new() -> Self {
        Self {
            is_open: true,
            current_game: None,
            returned_queue: VecDeque::new(),
            games_queue: VecDeque::new(),
            skipped: Vec::new(),
            raffle: Raffle::new(),
        }
    }
    fn queue(&self) -> impl Iterator<Item = &Game> {
        self.returned_queue.iter().chain(self.games_queue.iter())
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Raffle {
    viewers_weight: HashMap<String, usize>,
    mode: RaffleMode,
}

#[derive(Serialize, Deserialize, Clone)]
enum RaffleMode {
    Inactive,
    Active { joined: HashMap<String, usize> },
}

impl Raffle {
    fn new() -> Self {
        Self {
            viewers_weight: HashMap::new(),
            mode: RaffleMode::Inactive,
        }
    }
}

impl GameJamBot {
    pub fn name() -> &'static str {
        "GameJamBot"
    }
    pub fn new(channel: &String) -> Self {
        let config: GameJamConfig = serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open("config/gamejam/gamejam_config.json").unwrap(),
        ))
        .unwrap();

        let mut bot = Self {
            channel_login: channel.clone(),
            config,
            commands: Self::commands(),
            save_file: "config/gamejam/gamejam_nertsalbot.json".to_owned(),
            played_games_file: "config/gamejam/games_played.json".to_owned(),
            played_games: Vec::new(),
            games_state: GamesState::new(),
            time_limit: None,
            hub: None,
            update_sheets: false,
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

        println!("Loading GameJamBot data from {}", &bot.save_file);
        match bot.load_games() {
            Ok(_) => {
                println!("Successfully loaded GameJamBot data")
            }
            Err(error) => {
                use std::io::ErrorKind;
                match error.kind() {
                    ErrorKind::NotFound => {
                        println!("Using default GameJamBot data");
                        bot.save_games().unwrap();
                    }
                    _ => panic!("Error loading GameJamBot data: {}", error),
                }
            }
        }
        match load_from(&bot.played_games_file) {
            Ok(played_games) => bot.played_games = played_games,
            Err(error) => {
                use std::io::ErrorKind;
                match error.kind() {
                    ErrorKind::NotFound => {
                        save_into(&bot.played_games, &bot.played_games_file).unwrap();
                    }
                    _ => panic!("Error loading GameJamBot data: {}", error),
                }
            }
        }

        bot
    }
    fn check_message(&mut self, message: &PrivmsgMessage) -> Option<String> {
        if let Some(_) = self.time_limit {
            let game = self.games_state.current_game.as_ref().unwrap();
            if message.sender.name == game.author {
                self.time_limit = None;
                let reply = format!("Now playing {} from @{}. ", game.name, game.author);
                return Some(reply);
            }
        }
        None
    }
    fn update(&mut self) -> Option<String> {
        if let Some(time) = self.time_limit {
            if time.elapsed().as_secs() >= self.config.response_time_limit.unwrap() {
                self.time_limit = None;
                return self.skip(true);
            }
        }
        None
    }
    pub fn save_games(&mut self) -> std::io::Result<()> {
        self.update_sheets = true;
        save_into(&self.games_state, &self.save_file)
    }
    fn load_games(&mut self) -> std::io::Result<()> {
        self.games_state = load_from(&self.save_file)?;
        Ok(())
    }
    async fn save_sheets(&self) -> google_sheets4::Result<()> {
        use google_sheets4::api::*;

        let mut rows = Vec::new();
        rows.push(self.values_to_row_data(
            vec!["Game link".to_owned(), "Author".to_owned()],
            Some(CellFormat {
                text_format: Some(TextFormat {
                    bold: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        ));
        if let Some(game) = &self.games_state.current_game {
            rows.push(self.values_to_row_data(
                vec![game.name.clone(), game.author.clone()],
                self.game_to_format(GameType::Current),
            ));
        }
        for game in self.games_state.queue() {
            rows.push(self.values_to_row_data(
                vec![game.name.clone(), game.author.clone()],
                self.game_to_format(GameType::Queued),
            ));
        }
        for game in &self.games_state.skipped {
            rows.push(self.values_to_row_data(
                vec![game.name.clone(), game.author.clone()],
                self.game_to_format(GameType::Skipped),
            ));
        }
        for game in &self.played_games {
            rows.push(self.values_to_row_data(
                vec![game.name.clone(), game.author.clone()],
                self.game_to_format(GameType::Played),
            ));
        }

        let update_values = BatchUpdateSpreadsheetRequest {
            requests: Some(vec![
                Request {
                    repeat_cell: Some(RepeatCellRequest {
                        fields: Some("*".to_owned()),
                        range: Some(GridRange {
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                Request {
                    update_cells: Some(UpdateCellsRequest {
                        rows: Some(rows),
                        fields: Some("*".to_owned()),
                        start: Some(GridCoordinate {
                            row_index: Some(0),
                            column_index: Some(0),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };
        let result = self
            .hub
            .as_ref()
            .unwrap()
            .spreadsheets()
            .batch_update(
                update_values,
                &self.config.google_sheet_config.as_ref().unwrap().sheet_id,
            )
            .add_scope(Scope::Spreadsheet)
            .doit()
            .await;
        result.map(|_| ())
    }
    fn game_to_format(&self, game_type: GameType) -> Option<google_sheets4::api::CellFormat> {
        use google_sheets4::api::*;
        let cell_format = &self
            .config
            .google_sheet_config
            .as_ref()
            .unwrap()
            .cell_format;
        let color = match game_type {
            GameType::Queued => cell_format.color_queued.clone(),
            GameType::Current => cell_format.color_current.clone(),
            GameType::Skipped => cell_format.color_skipped.clone(),
            GameType::Played => cell_format.color_played.clone(),
        };
        Some(CellFormat {
            background_color: color,
            ..Default::default()
        })
    }
    fn values_to_row_data(
        &self,
        values: Vec<String>,
        user_entered_format: Option<google_sheets4::api::CellFormat>,
    ) -> google_sheets4::api::RowData {
        use google_sheets4::api::*;
        let mut cells = Vec::with_capacity(values.len());
        for value in values {
            cells.push(CellData {
                user_entered_value: Some(ExtendedValue {
                    string_value: Some(value),
                    ..Default::default()
                }),
                user_entered_format: user_entered_format.clone(),
                ..Default::default()
            });
        }
        RowData {
            values: Some(cells),
        }
    }
}

fn save_into<T: Serialize>(value: &T, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    serde_json::to_writer(file, value)?;
    Ok(())
}
fn load_from<T: serde::de::DeserializeOwned>(
    path: impl AsRef<std::path::Path>,
) -> std::io::Result<T> {
    let file = std::io::BufReader::new(std::fs::File::open(path)?);
    Ok(serde_json::from_reader(file)?)
}

#[async_trait]
impl Bot for GameJamBot {
    fn name(&self) -> &str {
        Self::name()
    }
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &ServerMessage,
    ) {
        if let Some(reply) = self.update() {
            send_message(client, self.channel_login.clone(), reply).await;
        }
        match message {
            ServerMessage::Privmsg(message) => {
                if let Some(reply) = self.check_message(message) {
                    send_message(client, self.channel_login.clone(), reply).await;
                }
                check_command(self, client, self.channel_login.clone(), message).await;
            }
            _ => (),
        };

        if self.update_sheets && self.config.google_sheet_config.is_some() {
            match self.save_sheets().await {
                Ok(_) => (),
                Err(err) => println!("Error trying to save queue into google sheets: {}", err),
            }
            self.update_sheets = false;
        }
    }
}
