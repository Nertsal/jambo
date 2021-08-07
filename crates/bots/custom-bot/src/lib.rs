use bot_core::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

mod commands;

#[derive(Clone, Serialize, Deserialize)]
struct CustomConfig {
    commands: HashMap<String, String>,
}

impl CustomConfig {
    fn save(&self) -> std::io::Result<()> {
        serde_json::to_writer(
            std::io::BufWriter::new(std::fs::File::create("config/custom/custom_config.json")?),
            self,
        )?;
        Ok(())
    }
    fn load() -> std::io::Result<CustomConfig> {
        Ok(serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open("config/custom/custom_config.json")?,
        ))?)
    }
}

#[derive(Bot)]
pub struct CustomBot {
    cli: CLI,
    config: CustomConfig,
    commands: Commands<Self, Sender>,
}

impl CustomBot {
    pub fn name() -> &'static str {
        "CustomBot"
    }

    pub fn new(cli: &CLI) -> Box<dyn Bot> {
        let config = match CustomConfig::load() {
            Ok(config) => config,
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => {
                    let config = CustomConfig {
                        commands: HashMap::new(),
                    };
                    config.save().unwrap();
                    config
                }
                _ => panic!("{}", error),
            },
        };
        let mut bot = Self {
            cli: Arc::clone(cli),
            commands: Self::commands(),
            config: config.clone(),
        };
        for (command_name, _) in config.commands {
            bot.push_command(command_name);
        }
        Box::new(bot)
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
