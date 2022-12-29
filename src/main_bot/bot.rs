use super::*;

pub struct MainBot {
    pub(super) cli: Option<Cli>,
    pub(super) commands: Commands<MainBot>,
    pub(super) bots: Bots,
    pub queue_shutdown: bool,
    users: HashMap<String, User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    color: Option<Color>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl MainBot {
    pub fn new(cli: Option<&Cli>, active_bots: ActiveBots) -> Self {
        Self {
            cli: cli.cloned(),
            commands: Self::commands(active_bots.iter().cloned()),
            bots: Bots::new(&cli.cloned(), active_bots),
            queue_shutdown: false,
            users: HashMap::new(),
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
                self.log_chat_message(&message);
                self.handle_message(
                    client,
                    &message.channel_login,
                    &private_to_command_message(&message),
                )
                .await;
            }
            ServerMessage::UserNotice(message) => {
                self.log(LogType::Event, &message.system_message);
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
        log(&self.cli, log_type, message)
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

    fn log_chat_message(&mut self, message: &twitch_irc::message::PrivmsgMessage) {
        use colored::Colorize;

        // Color the user's name
        let (sender_name, color) = match &message.name_color {
            Some(color) => (
                message
                    .sender
                    .name
                    .truecolor(color.r, color.g, color.b)
                    .to_string(),
                Some(Color {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                }),
            ),
            None => (message.sender.name.clone(), None),
        };
        // Register the user
        self.users
            .entry(message.sender.name.to_lowercase())
            .or_insert(User { color: None })
            .color = color;

        // Print the colored message
        if let Some(cli) = &self.cli {
            let colored =
                self.color_message(&format!("{}: {}", sender_name, &message.message_text));
            let mut writer = cli.lock_writer_erase().unwrap();
            write!(writer, "{}", LogType::Chat).unwrap();
            for word in colored {
                write!(writer, " {}", word).unwrap();
            }
            writeln!(writer).unwrap();
        }
    }

    fn color_message(&self, message: &str) -> Vec<colored::ColoredString> {
        use colored::Colorize;

        message
            .split_whitespace()
            .map(|word| {
                self.users
                    .get(&word.to_lowercase())
                    .and_then(|user| user.color)
                    .map(|color| word.truecolor(color.r, color.g, color.b))
                    .unwrap_or_else(|| word.clear())
            })
            .collect()
    }
}

impl BotPerformer for MainBot {
    const NAME: &'static str = "MainBot";

    fn commands(&self) -> &Commands<Self> {
        &self.commands
    }
}
