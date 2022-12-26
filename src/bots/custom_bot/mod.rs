use std::collections::HashMap;

use super::*;

mod commands;

pub struct CustomBot {
    cli: Option<Cli>,
    config: CustomConfig,
    commands: Commands<Self>,
}

#[derive(Debug, Serialize)]
pub struct CustomSerialized {
    config: CustomConfig,
}

impl CustomBot {
    pub fn new_boxed(cli: &Option<Cli>) -> Box<dyn Bot> {
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
            cli: cli.clone(),
            commands: Self::commands(),
            config: config.clone(),
        };
        for (command_name, _) in config.commands {
            bot.push_command(command_name);
        }
        Box::new(bot)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl BotPerformer for CustomBot {
    const NAME: &'static str = "CustomBot";

    fn commands(&self) -> &Commands<Self> {
        &self.commands
    }
}

#[async_trait]
impl Bot for CustomBot {
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
        SerializedBot::Custom(CustomSerialized {
            config: self.config.clone(),
        })
    }
}
