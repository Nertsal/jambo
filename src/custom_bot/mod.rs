use super::*;

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
    fn commands() -> BotCommands<Self> {
        BotCommands {
            commands: vec![
                BotCommand {
                    name: "command new".to_owned(),
                    authority_level: AuthorityLevel::Moderator,
                    command: |bot, _, _, args| {
                        let mut words = args.split_whitespace();
                        if let Some(command_name) = words.next() {
                            let command_response: String = words.collect();
                            if command_response.len() > 0 {
                                let response = Some(format!(
                                    "Added new command !{}: {}",
                                    command_name, command_response
                                ));
                                if bot.new_command(
                                    command_name.to_owned(),
                                    command_response.to_owned(),
                                ) {
                                    return response;
                                }
                            }
                        }
                        None
                    },
                },
                BotCommand {
                    name: "command remove".to_owned(),
                    authority_level: AuthorityLevel::Moderator,
                    command: |bot, _, _, args| {
                        if let Some(command_response) = bot.config.commands.remove(&args) {
                            let response =
                                Some(format!("Removed command {}: {}", args, command_response));
                            let com_index = bot
                                .commands
                                .commands
                                .iter()
                                .position(|com| com.name == args)
                                .unwrap();
                            bot.commands.commands.remove(com_index);
                            bot.config.save().unwrap();
                            return response;
                        }
                        None
                    },
                },
                BotCommand {
                    name: "command edit".to_owned(),
                    authority_level: AuthorityLevel::Moderator,
                    command: |bot, _, _, args| {
                        let mut words = args.split_whitespace();
                        if let Some(command_name) = words.next() {
                            let command_response: String = words.collect();
                            if command_response.len() > 0 {
                                if let Some(old_response) =
                                    bot.config.commands.get_mut(command_name)
                                {
                                    let response = Some(format!(
                                        "Edited command {}: {}. New command: {}",
                                        command_name, old_response, command_response
                                    ));
                                    bot.update_command(command_name.to_owned(), command_response);
                                    return response;
                                }
                            }
                        }
                        None
                    },
                },
            ],
        }
    }
    fn new_command(&mut self, command_name: String, command_response: String) -> bool {
        if self.config.commands.contains_key(&command_name) {
            false
        } else {
            self.update_command(command_name, command_response);
            true
        }
    }
    fn update_command(&mut self, command_name: String, command_response: String) {
        self.config
            .commands
            .insert(command_name.clone(), command_response.clone());
        self.push_command(command_name);
        self.config.save().unwrap();
    }
    fn push_command(&mut self, command_name: String) {
        self.commands.commands.push(BotCommand {
            name: command_name,
            authority_level: AuthorityLevel::Any,
            command: move |bot, _, command_name, _| {
                Some(bot.config.commands[&command_name].clone())
            },
        });
    }
}

impl CommandBot<CustomBot> for CustomBot {
    fn commands(&self) -> &BotCommands<Self> {
        &self.commands
    }
}

#[async_trait]
impl Bot for CustomBot {
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
