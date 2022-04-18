mod commands;

use std::collections::HashMap;

use super::*;

use bots::*;

// -- Modify this section to include a new bot into the main bot --

#[derive(Debug, Serialize)]
pub enum SerializedBot {
    // Insert here
    Custom(CustomSerialized),
    Quote(QuoteSerialized),
    Timer(TimerSerialized),
    Vote(VoteSerialized),
}

fn constructors() -> impl IntoIterator<Item = (BotName, BotConstructor)> {
    // Add a line below to make constructing the bot possible
    [
        // Insert here
        (CustomBot::NAME.to_owned(), CustomBot::new as _),
        (QuoteBot::NAME.to_owned(), QuoteBot::new as _),
        (TimerBot::NAME.to_owned(), TimerBot::new as _),
        (VoteBot::NAME.to_owned(), VoteBot::new as _),
    ]
}

// -- End of the section, do not modify anything below --

pub struct MutexBot(Mutex<MainBot>);

pub struct MainBot {
    cli: Cli,
    commands: Commands<MainBot>,
    bots: Bots,
    pub queue_shutdown: bool,
}

impl MainBot {
    pub fn new(cli: &Cli, active_bots: ActiveBots) -> Self {
        Self {
            cli: cli.clone(),
            commands: Self::commands(),
            bots: Bots::new(cli, active_bots),
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

    fn save_bots(&self) -> std::io::Result<()> {
        let active_bots = self.bots.active.keys().cloned().collect::<HashSet<_>>();
        let file = std::io::BufWriter::new(std::fs::File::create("config/active_bots.json")?);
        serde_json::to_writer(file, &active_bots)?;
        Ok(())
    }

    pub fn log(&self, log_type: LogType, message: &str) {
        let mut writer = self.cli.lock_writer_erase().unwrap();
        writeln!(writer, "{} {}", log_type, message).unwrap();
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

    pub fn serialize<'a>(&'a self) -> impl Iterator<Item = SerializedBot> + 'a {
        self.bots.active.values().map(|bot| bot.serialize())
    }
}

#[async_trait]
pub trait Bot: Send {
    async fn handle_message(
        &mut self,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: &CommandMessage,
    );

    async fn update(&mut self, client: &TwitchClient, channel: &ChannelLogin, delta_time: f32) {
        #![allow(unused_variables)]
    }

    fn complete(
        &self,
        word: &str,
        prompter: &Prompter,
        start: usize,
        end: usize,
    ) -> Option<Vec<linefeed::Completion>>;

    fn serialize(&self) -> SerializedBot;
}

#[async_trait]
pub trait BotPerformer {
    const NAME: &'static str;

    fn commands(&self) -> &Commands<Self>;

    async fn perform(
        &mut self,
        cli: &Cli,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: &CommandMessage,
    ) {
        let message_origin = &message.sender.origin;
        let commands = self.commands();
        let matched = commands.find_commands(message).collect::<Vec<_>>();
        for (command, args) in matched {
            if let Some(command_reply) = command(self, &message.sender, args) {
                match message_origin {
                    MessageOrigin::Twitch => {
                        send_message(cli, client, channel.clone(), command_reply).await;
                    }
                    MessageOrigin::Console => {
                        log(cli, LogType::Console, &command_reply);
                    }
                }
            }
        }
    }

    /// Write bot's status into a status file
    fn update_status(&self, status_text: &str) {
        let path = format!("status/{}.txt", Self::NAME);
        std::fs::write(path, status_text).expect("Could not update bot status");
    }
}

impl MutexBot {
    pub fn new(bot: MainBot) -> Self {
        Self(Mutex::new(bot))
    }
}

impl std::ops::Deref for MutexBot {
    type Target = Mutex<MainBot>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

type BotConstructor = fn(&Cli) -> Box<dyn Bot>;

struct Bots {
    constructors: HashMap<BotName, BotConstructor>,
    active: HashMap<BotName, Box<dyn Bot>>,
}

impl Bots {
    fn new(cli: &Cli, active_bots: ActiveBots) -> Self {
        let constructors = constructors().into_iter().collect::<HashMap<_, _>>();
        let mut active = HashMap::new();
        for bot_name in active_bots {
            match constructors.get(&bot_name) {
                Some(constructor) => {
                    let bot = constructor(cli);
                    log(cli, LogType::Info, &format!("Spawned {bot_name}"));
                    active.insert(bot_name, bot);
                }
                None => {
                    log(
                        cli,
                        LogType::Warn,
                        &format!("Failed to find a constructor for {bot_name}"),
                    );
                }
            }
        }
        Self {
            constructors,
            active,
        }
    }
}

impl BotPerformer for MainBot {
    const NAME: &'static str = "MainBot";

    fn commands(&self) -> &Commands<Self> {
        &self.commands
    }
}

impl linefeed::Completer<linefeed::DefaultTerminal> for MutexBot {
    fn complete(
        &self,
        word: &str,
        prompter: &Prompter,
        start: usize,
        end: usize,
    ) -> Option<Vec<linefeed::Completion>> {
        let mut main = futures::executor::block_on(self.0.lock());
        let main_completetion = main.commands.complete(word, prompter, start, end);
        let bots = &mut main.bots;

        let mut completions = vec![main_completetion];
        completions.extend(
            bots.active
                .values()
                .map(|bot| bot.complete(word, prompter, start, end)),
        );

        Some(completions.into_iter().flatten().flatten().collect())
    }
}
