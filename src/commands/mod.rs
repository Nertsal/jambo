use super::*;

mod command_message;
mod command_node;

pub use command_message::*;
pub use command_node::*;

pub trait CommandBot<T> {
    fn get_commands(&self) -> &BotCommands<T>;
}

pub async fn check_command<T: CommandBot<T>>(
    bot: &mut T,
    client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
    channel_login: String,
    message: &CommandMessage,
) {
    for (command, args) in bot.get_commands().find_commands(message) {
        if let Some(command_reply) = command(bot, message.sender_name.clone(), args) {
            send_message(client, channel_login.clone(), command_reply).await;
        }
    }
}

pub struct BotCommands<T> {
    pub commands: Vec<CommandNode<T>>,
}

pub enum AuthorityLevel {
    Broadcaster,
    Moderator,
    Any,
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

fn check_authority(authority_level: &AuthorityLevel, message: &CommandMessage) -> bool {
    match authority_level {
        AuthorityLevel::Any => true,
        AuthorityLevel::Broadcaster => check_badges(vec!["broadcaster"], message),
        AuthorityLevel::Moderator => check_badges(vec!["broadcaster", "moderator"], message),
    }
}

fn check_badges(badges: Vec<&str>, message: &CommandMessage) -> bool {
    message
        .badges
        .iter()
        .any(|badge| badges.contains(&badge.name.as_str()))
}
