use super::*;
use nertsal_commands::Commands;

pub trait CommandBot<T: Sync + Send, S>: Sync {
    fn get_commands(&self) -> &Commands<T, S>;

    fn get_cli(&self) -> &CLI;
}

pub async fn perform_commands<T: CommandBot<T, Sender> + Sync + Send>(
    bot: &mut T,
    client: &TwitchClient,
    channel_login: String,
    message: &CommandMessage<Sender>,
) {
    let message_origin = message.sender.origin;
    let commands = bot.get_commands();
    for (command, args) in commands.find_commands(message) {
        if let Some(command_reply) = command(bot, &message.sender, args) {
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
