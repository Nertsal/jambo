use super::*;

pub struct MainBot {
    pub(super) cli: Option<Cli>,
    pub(super) commands: Commands<MainBot>,
    pub(super) bots: Bots,
    pub queue_shutdown: bool,
}

impl MainBot {
    pub fn new(cli: Option<&Cli>, active_bots: ActiveBots) -> Self {
        Self {
            cli: cli.cloned(),
            commands: Self::commands(active_bots.iter().cloned()),
            bots: Bots::new(&cli.cloned(), active_bots),
            queue_shutdown: false,
        }
    }

    pub async fn handle_server_message(&mut self, client: &TwitchClient, message: ServerMessage) {
        match message {
            ServerMessage::Join(message) => {
                log(
                    &self.cli,
                    LogType::Info,
                    &format!("Joined {}", message.channel_login),
                );
            }
            ServerMessage::Notice(message) => {
                if message.message_text == "Login authentication failed" {
                    panic!("Login authentication failed.");
                }
            }
            ServerMessage::Privmsg(message) => {
                use colored::*;
                let sender_name = match &message.name_color {
                    Some(color) => message
                        .sender
                        .name
                        .truecolor(color.r, color.g, color.b)
                        .to_string(),
                    None => message.sender.name.clone(),
                };
                log(
                    &self.cli,
                    LogType::Chat,
                    &format!("{}: {}", sender_name, message.message_text),
                );
                self.handle_message(
                    client,
                    &message.channel_login,
                    &private_to_command_message(&message),
                )
                .await;
            }
            _ => (),
        }
    }

    pub(super) fn save_bots(&self) -> std::io::Result<()> {
        let active_bots = self.bots.active.keys().cloned().collect::<HashSet<_>>();
        let file = std::io::BufWriter::new(std::fs::File::create("config/active_bots.json")?);
        serde_json::to_writer(file, &active_bots)?;
        Ok(())
    }

    pub fn log(&self, log_type: LogType, message: &str) {
        if let Some(cli) = &self.cli {
            let mut writer = cli.lock_writer_erase().unwrap();
            writeln!(writer, "{} {}", log_type, message).unwrap();
        }
    }

    pub async fn handle_message(
        &mut self,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: &CommandMessage,
    ) {
        self.perform(&self.cli.clone(), client, channel, message)
            .await;

        for bot in self.bots.active.values_mut() {
            bot.handle_message(client, channel, message).await;
        }
    }

    pub async fn update(&mut self, client: &TwitchClient, channel: &ChannelLogin, delta_time: f32) {
        for bot in self.bots.active.values_mut() {
            bot.update(client, channel, delta_time).await;
        }
    }

    pub fn serialize(&self) -> impl Iterator<Item = SerializedBot> + '_ {
        self.bots.active.values().map(|bot| bot.serialize())
    }
}

impl BotPerformer for MainBot {
    const NAME: &'static str = "MainBot";

    fn commands(&self) -> &Commands<Self> {
        &self.commands
    }
}
