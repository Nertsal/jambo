use super::*;

mod commands;

#[derive(Clone, Serialize, Deserialize)]
struct CustomConfig {
    commands: HashMap<String, String>,
}

impl CustomConfig {
    fn save(&self) -> std::io::Result<()> {
        serde_json::to_writer(
            std::io::BufWriter::new(std::fs::File::create("config/custom/custom-config.json")?),
            self,
        )?;
        Ok(())
    }
    fn load() -> std::io::Result<CustomConfig> {
        Ok(serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open("config/custom/custom-config.json")?,
        ))?)
    }
}

pub struct CustomBot {
    channel_login: String,
    config: CustomConfig,
    commands: BotCommands<Self>,
}

impl CustomBot {
    pub fn name() -> &'static str {
        "CustomBot"
    }
    pub fn new(channel_login: &String) -> Self {
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
                _ => panic!(error),
            },
        };
        let mut bot = Self {
            channel_login: channel_login.clone(),
            commands: Self::commands(),
            config: config.clone(),
        };
        for (command_name, _) in config.commands {
            bot.push_command(command_name);
        }
        bot
    }
}

#[async_trait]
impl Bot for CustomBot {
    fn name(&self) -> &str {
        Self::name()
    }
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
