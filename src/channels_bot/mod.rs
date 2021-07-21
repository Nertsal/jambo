use std::collections::HashSet;

use super::*;

mod commands;

pub type ActiveBots = HashSet<String>;

pub struct ChannelsBot {
    channel_login: String,
    cli: CLI,
    pub queue_shutdown: bool,
    commands: BotCommands<Self>,
    available_bots: HashMap<String, Box<fn(&CLI, &str) -> Box<dyn Bot>>>,
    active_bots: HashMap<String, Box<dyn Bot>>,
}

impl ChannelsBot {
    pub fn name() -> &'static str {
        "ChannelsBot"
    }

    pub fn new(cli: &CLI, config: &LoginConfig, active_bots: &ActiveBots) -> Box<Self> {
        let mut bot = Self {
            channel_login: config.channel_login.clone(),
            cli: Arc::clone(&cli),
            queue_shutdown: false,
            commands: Self::commands(),
            available_bots: Self::available_bots(),
            active_bots: HashMap::with_capacity(active_bots.len()),
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

    async fn handle_server_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &ServerMessage,
    ) {
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
                    &format!(
                        "{} {}: {}",
                        message.channel_login, sender_name, message.message_text
                    ),
                );
                let channel_login = self.channel_login.clone();
                check_command(self, client, channel_login, &CommandMessage::from(message)).await;
            }
            _ => (),
        }
        for bot in self.active_bots.values_mut() {
            bot.handle_server_message(client, &message).await;
        }
    }

    async fn handle_command_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &CommandMessage,
    ) {
        let channel_login = self.channel_login.clone();
        check_command(self, client, channel_login, message).await;

        for bot in self.active_bots.values_mut() {
            bot.handle_command_message(client, message).await;
        }
    }

    async fn update(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        delta_time: f32,
    ) {
        for bot in self.active_bots.values_mut() {
            bot.update(client, delta_time).await;
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
