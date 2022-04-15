mod commands;

use std::collections::HashMap;

use super::*;

use bots::*;

// -- Modify this section to include a new bot into the main bot

fn constructors() -> impl IntoIterator<Item = (BotName, BotConstructor)> {
    // Add a line below to make constructing the bot possible
    [
        // Insert here
        (CustomBot::NAME.to_owned(), CustomBot::subbot as _),
    ]
}

// -- End of the section, do not modify anything below

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

        // To include the sub-bot into auto-completion, add a line in the match below
        let mut completions = vec![main_completetion];
        completions.extend(
            bots.active
                .values()
                .map(|bot| bot.complete(word, prompter, start, end)),
        );

        Some(completions.into_iter().flatten().flatten().collect())
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

    fn complete(
        &self,
        word: &str,
        prompter: &Prompter,
        start: usize,
        end: usize,
    ) -> Option<Vec<linefeed::Completion>>;
}

pub struct MutexBot(Mutex<MainBot>);

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
                        &format!("Failed to find a constructor for bot named {bot_name}"),
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

pub struct MainBot {
    cli: Cli,
    commands: Commands<MainBot>,
    bots: Bots,
}

impl BotPerformer for MainBot {
    fn commands(&self) -> &Commands<Self> {
        &self.commands
    }
}

#[async_trait]
impl Bot for MainBot {
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
}

impl MainBot {
    pub fn new(cli: &Cli, active_bots: ActiveBots) -> Self {
        Self {
            cli: cli.clone(),
            commands: Self::commands(),
            bots: Bots::new(cli, active_bots),
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

    pub async fn update(&mut self, client: &TwitchClient, channel: &ChannelLogin, delta_time: f32) {
        // for bot in self.active_bots.values_mut() {
        //     bot.update(client, channel, delta_time).await;
        // }
    }

    pub fn log(&self, log_type: LogType, message: &str) {
        let mut writer = self.cli.lock_writer_erase().unwrap();
        writeln!(writer, "{} {}", log_type, message).unwrap();
    }
}

#[async_trait]
pub trait BotPerformer: Bot {
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
}
