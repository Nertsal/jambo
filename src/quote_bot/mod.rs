use super::*;

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
    fn commands() -> BotCommands<Self> {
        BotCommands {
            commands: vec![
                BotCommand {
                    name: "quote add".to_owned(),
                    authority_level: AuthorityLevel::Moderator,
                    command: |bot, _, args| {
                        let quote_id = bot.config.id_generator.gen();
                        let response =
                            Some(format!("Added new quote {}: {}", quote_id.raw(), args));
                        bot.config.quotes.insert(quote_id, args);
                        bot.config.save().unwrap();
                        response
                    },
                },
                BotCommand {
                    name: "quote delete".to_owned(),
                    authority_level: AuthorityLevel::Moderator,
                    command: |bot, _, args| {
                        if let Ok(quote_id) = serde_json::from_str(args.as_str()) {
                            if let Some(quote) = bot.config.quotes.remove(&quote_id) {
                                let response =
                                    Some(format!("Deleted quote {:?}: {}", quote_id.raw(), quote));
                                bot.config.save().unwrap();
                                return response;
                            }
                        }
                        None
                    },
                },
                BotCommand {
                    name: "quote replace".to_owned(),
                    authority_level: AuthorityLevel::Moderator,
                    command: |bot, _, args| {
                        let mut words = args.split_whitespace();
                        if let Some(quote_id) = words.next() {
                            let args = words.collect();
                            if let Ok(quote_id) = serde_json::from_str(quote_id) {
                                let response =
                                    if let Some(quote) = bot.config.quotes.get_mut(&quote_id) {
                                        let response = Some(format!(
                                            "Replaced quote {}: {}. New quote: {}",
                                            quote_id.raw(),
                                            quote,
                                            args
                                        ));
                                        *quote = args;
                                        response
                                    } else {
                                        let response = Some(format!(
                                            "Added new quote {}: {}",
                                            quote_id.raw(),
                                            args
                                        ));
                                        bot.config.quotes.insert(quote_id, args);
                                        response
                                    };
                                bot.config.save().unwrap();
                                return response;
                            }
                        }
                        None
                    },
                },
                BotCommand {
                    name: "quote".to_owned(),
                    authority_level: AuthorityLevel::Any,
                    command: |bot, _, args| {
                        if let Ok(quote_id) = serde_json::from_str(args.as_str()) {
                            if let Some(quote) = bot.config.quotes.get(&quote_id) {
                                let response = Some(quote.clone());
                                return response;
                            }
                        }
                        None
                    },
                },
            ],
        }
    }
}

impl CommandBot<QuoteBot> for QuoteBot {
    fn commands(&self) -> &BotCommands<Self> {
        &self.commands
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
