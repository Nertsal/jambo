mod commands;

use std::collections::HashMap;

use super::*;

use bots::*;

// -- Modify this section to include a new bot into the main bot

// List all sub-bots in this enum,
// then add a line in the constructor,
// add 2 lines for each bot in the functions below
// to include it in performance and autocompletion
pub enum SubBot {
    // Insert here
    Custom(CustomBot),
}

fn constructors() -> impl IntoIterator<Item = (BotName, BotConstructor)> {
    // Add a line below to make constructing the bot possible
    [
        // Insert here
        (CustomBot::NAME.to_owned(), CustomBot::subbot as _),
    ]
}

impl MainBot {
    pub async fn handle_command_message(
        &mut self,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: CommandMessage,
    ) {
        let cli = &self.cli.clone();
        self.perform(cli, client, channel, &message).await;

        // To make the sub-bot perform commands, add a line in the match below
        for bot in self.bots.active.values_mut() {
            match bot {
                // Insert here
                SubBot::Custom(bot) => bot.perform(cli, client, channel, &message).await,
            }
        }
    }
}

impl<Term: linefeed::Terminal> linefeed::Completer<Term> for MutexBot {
    fn complete(
        &self,
        word: &str,
        prompter: &linefeed::Prompter<Term>,
        start: usize,
        end: usize,
    ) -> Option<Vec<linefeed::Completion>> {
        let mut main = futures::executor::block_on(self.0.lock());
        let main_completetion = main.commands.complete(word, prompter, start, end);
        let bots = &mut main.bots;

        // To include the sub-bot into auto-completion, add a line in the match below
        let mut completions = vec![main_completetion];
        completions.extend(bots.active.values().map(|bot| match bot {
            // Insert here
            SubBot::Custom(bot) => bot.complete(word, prompter, start, end),
        }));

        Some(completions.into_iter().flatten().flatten().collect())
    }
}

// -- End of the section, do not modify anything below

pub trait Bot<T> {
    const NAME: &'static str;

    fn inner(&mut self) -> &mut T;
    fn commands(&self) -> &Commands<T>;

    fn complete<Term: linefeed::Terminal>(
        &self,
        word: &str,
        prompter: &linefeed::Prompter<Term>,
        start: usize,
        end: usize,
    ) -> Option<Vec<linefeed::Completion>> {
        self.commands().complete(word, prompter, start, end)
    }
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

type BotConstructor = fn(&Cli) -> SubBot;

struct Bots {
    constructors: HashMap<BotName, BotConstructor>,
    active: HashMap<BotName, SubBot>,
}

impl Bots {
    fn new(cli: &Cli, active_bots: ActiveBots) -> Self {
        let constructors = constructors().into_iter().collect::<HashMap<_, _>>();
        let mut active = HashMap::new();
        for bot_name in active_bots {
            let bot = constructors[&bot_name](cli);
            active.insert(bot_name, bot);
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

impl Bot<Self> for MainBot {
    const NAME: &'static str = "MainBot";

    fn inner(&mut self) -> &mut Self {
        self
    }

    fn commands(&self) -> &Commands<Self> {
        &self.commands
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
                self.handle_command_message(
                    client,
                    &message.channel_login,
                    private_to_command_message(&message),
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
trait BotPerformer<T> {
    async fn perform(
        &mut self,
        cli: &Cli,
        client: &TwitchClient,
        channel: &ChannelLogin,
        message: &CommandMessage,
    );
}

#[async_trait]
impl<T: Send, B: Bot<T> + Send> BotPerformer<T> for B {
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
            if let Some(command_reply) = command(self.inner(), &message.sender, args) {
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
