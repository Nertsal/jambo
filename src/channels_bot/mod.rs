use std::collections::HashSet;

use super::*;

mod commands;

#[derive(Clone, Serialize, Deserialize)]
pub struct BotsConfig {
    active_bots: HashSet<String>,
}

pub struct ChannelsBot {
    channel_login: String,
    commands: BotCommands<Self>,
    available_bots: HashMap<String, Box<fn(&str) -> Box<dyn Bot>>>,
    active_bots: HashMap<String, Box<dyn Bot>>,
}

impl ChannelsBot {
    pub fn new(config: &LoginConfig, bots_config: &BotsConfig) -> Box<Self> {
        let mut bot = Self {
            channel_login: config.channel_login.clone(),
            commands: Self::commands(),
            available_bots: Self::available_bots(),
            active_bots: HashMap::with_capacity(bots_config.active_bots.len()),
        };
        for active_bot in &bots_config.active_bots {
            bot.spawn_bot(active_bot);
        }
        Box::new(bot)
    }

    pub async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: ServerMessage,
    ) {
        match &message {
            ServerMessage::Join(message) => {
                println!("Joined: {}", message.channel_login);
            }
            ServerMessage::Notice(message) => {
                if message.message_text == "Login authentication failed" {
                    panic!("Login authentication failed.");
                }
            }
            ServerMessage::Privmsg(message) => {
                println!(
                    "Got a message in channel {} from {}: {}",
                    message.channel_login, message.sender.name, message.message_text
                );
                let channel_login = self.channel_login.clone();
                check_command(self, client, channel_login, message).await;
            }
            _ => (),
        }
        for bot in self.active_bots.values_mut() {
            bot.handle_message(client, &message).await;
        }
    }

    pub async fn update(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        delta_time: f32,
    ) {
        for bot in self.active_bots.values_mut() {
            bot.update(client, delta_time).await;
        }
    }
}
