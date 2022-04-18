use std::collections::HashMap;

use super::*;

mod commands;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct QuoteConfig {
    quotes: HashMap<String, String>,
}

pub struct QuoteBot {
    cli: Cli,
    config: QuoteConfig,
    commands: Commands<Self>,
}

#[derive(Debug, Serialize)]
pub struct QuoteSerialized {
    config: QuoteConfig,
}

impl QuoteBot {
    pub fn new(cli: &Cli) -> Box<dyn Bot> {
        let config = match QuoteConfig::load() {
            Ok(config) => config,
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => {
                    let config = QuoteConfig::default();
                    config.save().unwrap();
                    config
                }
                _ => panic!("{}", error),
            },
        };
        Box::new(Self {
            cli: Arc::clone(cli),
            config,
            commands: Self::commands(),
        })
    }
}

impl QuoteConfig {
    fn save(&self) -> std::io::Result<()> {
        serde_json::to_writer(
            std::io::BufWriter::new(std::fs::File::create("config/quote/quote_config.json")?),
            self,
        )?;
        Ok(())
    }
    fn load() -> std::io::Result<Self> {
        Ok(serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open("config/quote/quote_config.json")?,
        ))?)
    }
}

impl BotPerformer for QuoteBot {
    const NAME: &'static str = "QuoteBot";

    fn commands(&self) -> &Commands<Self> {
        &self.commands
    }
}

#[async_trait]
impl Bot for QuoteBot {
    async fn handle_message(
        &mut self,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: &CommandMessage,
    ) {
        self.perform(&self.cli.clone(), client, channel, message)
            .await;
    }

    fn complete(
        &self,
        word: &str,
        prompter: &Prompter,
        start: usize,
        end: usize,
    ) -> Option<Vec<linefeed::Completion>> {
        self.commands.complete(word, prompter, start, end)
    }

    fn serialize(&self) -> SerializedBot {
        SerializedBot::Quote(QuoteSerialized {
            config: self.config.clone(),
        })
    }
}
