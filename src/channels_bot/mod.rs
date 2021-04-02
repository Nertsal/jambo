use super::*;

mod commands;

#[derive(Clone, Serialize, Deserialize)]
pub struct BotsConfig {
    ludumdare: bool,
    reply: bool,
    quote: bool,
    custom: bool,
}

pub struct ChannelsBot {
    channel: String,
    commands: BotCommands<ChannelsBot>,
    bots: HashMap<String, Box<dyn Bot>>,
}

impl CommandBot<ChannelsBot> for ChannelsBot {
    fn commands(&self) -> &BotCommands<ChannelsBot> {
        &self.commands
    }
}

impl ChannelsBot {
    pub fn new(config: &Config, bots_config: &BotsConfig) -> Self {
        let mut bot = Self {
            channel: config.channel.clone(),
            commands: Self::commands(),
            bots: HashMap::new(),
        };
        if bots_config.ludumdare {
            bot.spawn_bot("ludumdare");
        }
        if bots_config.reply {
            bot.spawn_bot("reply");
        }
        if bots_config.quote {
            bot.spawn_bot("quote");
        }
        if bots_config.custom {
            bot.spawn_bot("custom");
        }
        bot
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
                check_command(self, client, self.channel.clone(), message).await;
            }
            _ => (),
        }
        for bot in self.bots.values_mut() {
            bot.handle_message(client, &message).await;
        }
    }
}
