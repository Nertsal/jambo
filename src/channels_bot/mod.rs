use super::*;

mod commands;

#[derive(Clone, Serialize, Deserialize)]
pub struct BotsConfig {
    gamejam: bool,
    reply: bool,
    quote: bool,
    custom: bool,
    vote: bool,
    timer: bool,
}

pub struct ChannelsBot {
    channel_login: String,
    commands: BotCommands<Self>,
    bots: HashMap<String, Box<dyn Bot>>,
}

impl ChannelsBot {
    pub fn new(config: &LoginConfig, bots_config: &BotsConfig) -> Self {
        let mut bot = Self {
            channel_login: config.channel_login.clone(),
            commands: Self::commands(),
            bots: HashMap::new(),
        };
        if bots_config.gamejam {
            bot.spawn_bot(GameJamBot::name());
        }
        if bots_config.reply {
            bot.spawn_bot(ReplyBot::name());
        }
        if bots_config.quote {
            bot.spawn_bot(QuoteBot::name());
        }
        if bots_config.custom {
            bot.spawn_bot(CustomBot::name());
        }
        if bots_config.vote {
            bot.spawn_bot(VoteBot::name());
        }
        if bots_config.timer {
            bot.spawn_bot(TimerBot::name());
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
                let channel_login = self.channel_login.clone();
                check_command(self, client, channel_login, message).await;
            }
            _ => (),
        }
        for bot in self.bots.values_mut() {
            bot.handle_message(client, &message).await;
        }
    }
    pub async fn update(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        delta_time: f32,
    ) {
        for bot in self.bots.values_mut() {
            bot.update(client, delta_time).await;
        }
    }
}
