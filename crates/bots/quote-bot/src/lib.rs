use bot_core::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

mod commands;

#[derive(Serialize, Deserialize)]
struct QuoteConfig {
    quotes: HashMap<String, String>,
}

impl Default for QuoteConfig {
    fn default() -> Self {
        Self {
            quotes: HashMap::new(),
        }
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

#[derive(Bot)]
pub struct QuoteBot {
    cli: CLI,
    config: QuoteConfig,
    commands: Commands<Self, Sender>,
}

impl QuoteBot {
    pub fn name() -> &'static str {
        "QuoteBot"
    }

    pub fn new(cli: &CLI) -> Box<dyn Bot> {
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

    #[allow(unused_variables)]
    async fn handle_update(
        &mut self,
        client: &TwitchClient,
        channel_login: &String,
        delta_time: f32,
    ) {
    }
}
