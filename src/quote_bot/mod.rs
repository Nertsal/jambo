use super::*;

mod commands;

#[derive(Serialize, Deserialize)]
struct QuoteConfig {
    id_generator: IdGenerator,
    quotes: HashMap<Id, String>,
}

impl Default for QuoteConfig {
    fn default() -> Self {
        let mut id_generator = IdGenerator::new();
        id_generator.gen();
        Self {
            id_generator,
            quotes: HashMap::new(),
        }
    }
}

impl QuoteConfig {
    fn save(&self) -> std::io::Result<()> {
        serde_json::to_writer(
            std::io::BufWriter::new(std::fs::File::create("config/quote/quote-config.json")?),
            self,
        )?;
        Ok(())
    }
    fn load() -> std::io::Result<Self> {
        Ok(serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open("config/quote/quote-config.json")?,
        ))?)
    }
}

pub struct QuoteBot {
    channel_login: String,
    config: QuoteConfig,
    commands: BotCommands<Self>,
}

impl QuoteBot {
    pub fn new(channel_login: &String) -> Self {
        let config = match QuoteConfig::load() {
            Ok(config) => config,
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => {
                    let config = QuoteConfig::default();
                    config.save().unwrap();
                    config
                }
                _ => panic!(error),
            },
        };
        Self {
            channel_login: channel_login.clone(),
            config,
            commands: Self::commands(),
        }
    }
}

#[async_trait]
impl Bot for QuoteBot {
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &ServerMessage,
    ) {
        match message {
            ServerMessage::Privmsg(message) => {
                check_command(self, client, self.channel_login.clone(), message).await;
            }
            _ => (),
        };
    }
}
