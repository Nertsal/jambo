use super::*;
use std::collections::HashSet;

mod commands;

pub type ActiveBots = HashSet<String>;
type NewBotFn = Box<fn(&CLI) -> Box<dyn Bot>>;

pub struct ChannelsBot {
    commands: Commands<Self, Sender>,
    cli: CLI,
    pub queue_shutdown: bool,
    available_bots: HashMap<String, NewBotFn>,
    active_bots: HashMap<String, Box<dyn Bot>>,
}

impl ChannelsBot {
    pub fn name() -> &'static str {
        "ChannelsBot"
    }

    pub fn new(cli: &CLI, active_bots: &ActiveBots) -> Box<Self> {
        let available_bots = Self::available_bots();
        let mut bot = Self {
            cli: Arc::clone(&cli),
            queue_shutdown: false,
            active_bots: HashMap::with_capacity(active_bots.len()),
            commands: Self::commands(available_bots.keys()),
            available_bots,
        };
        for active_bot in active_bots {
            bot.spawn_bot(active_bot);
        }
        Box::new(bot)
    }
}

#[async_trait]
impl Bot for ChannelsBot {
    fn name(&self) -> &str {
        Self::name()
    }

    async fn handle_server_message(&mut self, client: &TwitchClient, message: &ServerMessage) {
        match message {
            ServerMessage::Join(message) => {
                self.log(LogType::Info, &format!("Joined {}", message.channel_login));
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
                self.log(
                    LogType::ChatMessage,
                    &format!("{}: {}", sender_name, message.message_text),
                );
                perform_commands(
                    self,
                    client,
                    message.channel_login.clone(),
                    &private_to_command_message(message),
                )
                .await;
            }
            _ => (),
        }
        for bot in self.active_bots.values_mut() {
            bot.handle_server_message(client, &message).await;
        }
    }

    async fn handle_command_message(
        &mut self,
        client: &TwitchClient,
        channel_login: &String,
        message: &CommandMessage<Sender>,
    ) {
        perform_commands(self, client, channel_login.clone(), message).await;

        for bot in self.active_bots.values_mut() {
            bot.handle_command_message(client, channel_login, message)
                .await;
        }
    }

    async fn update(&mut self, client: &TwitchClient, channel_login: &String, delta_time: f32) {
        for bot in self.active_bots.values_mut() {
            bot.update(client, channel_login, delta_time).await;
        }
    }

    fn get_completion_tree(&self) -> Vec<CompletionNode> {
        let mut completions = Vec::new();
        completions.append(&mut commands_to_completion(&self.get_commands().commands));
        for bot in self.active_bots.values() {
            completions.append(&mut bot.get_completion_tree());
        }
        completions
    }
}
