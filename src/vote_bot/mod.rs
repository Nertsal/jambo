use super::*;

mod commands;

pub struct VoteBot {
    channel_login: String,
    cli: CLI,
    commands: BotCommands<Self>,
    vote_mode: VoteMode,
}

impl VoteBot {
    pub fn name() -> &'static str {
        "VoteBot"
    }

    pub fn new(cli: &CLI, channel_login: &str) -> Box<dyn Bot> {
        Box::new(Self {
            channel_login: channel_login.to_owned(),
            cli: Arc::clone(cli),
            commands: Self::commands(),
            vote_mode: VoteMode::Inactive,
        })
    }
}

pub enum VoteMode {
    Inactive,
    Active { votes: HashMap<String, String> },
}

#[async_trait]
impl Bot for VoteBot {
    fn name(&self) -> &str {
        Self::name()
    }

    async fn handle_server_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &ServerMessage,
    ) {
        match message {
            ServerMessage::Privmsg(message) => {
                check_command(
                    self,
                    client,
                    self.channel_login.clone(),
                    &CommandMessage::from(message),
                )
                .await;
            }
            _ => (),
        };
    }

    async fn handle_command_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &CommandMessage,
    ) {
        check_command(self, client, self.channel_login.clone(), &message).await;
    }
}
