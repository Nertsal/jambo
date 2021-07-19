use super::*;

mod command_message;
mod command_node;

pub use command_message::*;
pub use command_node::*;

pub trait CommandBot<T: Sync + Send> {
    fn get_commands(&self) -> &BotCommands<T>;

    fn get_cli(&self) -> &CLI;
}

pub async fn check_command<T: CommandBot<T> + Sync + Send>(
    bot: &mut T,
    client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
    channel_login: String,
    message: &CommandMessage,
) {
    let message_origin = message.origin;
    for (command, args) in bot.get_commands().find_commands(message) {
        if let Some(command_reply) = command(bot, message.sender_name.clone(), args) {
            match message_origin {
                MessageOrigin::Chat => {
                    bot.send_message(client, channel_login.clone(), command_reply)
                        .await;
                }
                MessageOrigin::Console => {
                    bot.log(LogType::ConsoleResponse, &command_reply);
                }
            }
        }
    }
}

pub struct BotCommands<T> {
    pub commands: Vec<CommandNode<T>>,
}

impl<T> BotCommands<T> {
    fn find_commands(&self, message: &CommandMessage) -> Vec<(Command<T>, Vec<Argument>)> {
        self.commands
            .iter()
            .filter_map(|com| com.check_node(&message.message_text, Vec::new()))
            .filter_map(|(command, arguments)| match command {
                CommandNode::FinalNode {
                    authority_level,
                    command,
                } => {
                    if check_authority(authority_level, &message) {
                        Some((command.clone(), arguments))
                    } else {
                        None
                    }
                }
                _ => unreachable!(),
            })
            .collect()
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub enum AuthorityLevel {
    Viewer = 0,
    Moderator = 1,
    Broadcaster = 2,
}

impl AuthorityLevel {
    pub fn from_badges(badges: &Vec<twitch_irc::message::Badge>) -> Self {
        badges
            .iter()
            .fold(AuthorityLevel::Viewer, |authority_level, badge| {
                authority_level.max(AuthorityLevel::from_badge(badge))
            })
    }

    pub fn from_badge(badge: &twitch_irc::message::Badge) -> Self {
        match badge.name.as_str() {
            "broadcaster" => AuthorityLevel::Broadcaster,
            "moderator" => AuthorityLevel::Moderator,
            _ => AuthorityLevel::Viewer,
        }
    }
}

fn check_authority(authority_level: &AuthorityLevel, message: &CommandMessage) -> bool {
    message.authority_level >= *authority_level
}
