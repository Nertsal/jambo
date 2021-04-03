use super::*;
use std::collections::HashSet;

#[derive(Deserialize)]
struct ReplyConfig {
    responses: Vec<Response>,
}

pub struct ReplyBot {
    responses: Vec<Response>,
    channel_login: String,
}

impl ReplyBot {
    pub fn new(channel: &String) -> Self {
        let config: ReplyConfig = serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open("config/reply/reply-config.json").unwrap(),
        ))
        .unwrap();

        Self {
            responses: config.responses,
            channel_login: channel.clone(),
        }
    }
    fn check_response(&self, message: &PrivmsgMessage) -> Option<String> {
        let message_text = &message.message_text;
        if let Some(response) = self.responses.iter().find(|response| {
            let mut fits = true;
            for keywords in &response.keywords {
                if !keywords
                    .iter()
                    .any(|keyword| message_text.contains(keyword))
                {
                    fits = false;
                    break;
                }
            }
            fits
        }) {
            return Some(response.response.clone());
        }
        None
    }
}

#[async_trait]
impl Bot for ReplyBot {
    fn name(&self) -> &str {
        "ReplyBot"
    }
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
        message: &ServerMessage,
    ) {
        match message {
            ServerMessage::Privmsg(message) => {
                if let Some(response) = self.check_response(message) {
                    send_message(client, self.channel_login.clone(), response).await;
                }
            }
            _ => (),
        }
    }
}

#[derive(Deserialize)]
struct Response {
    keywords: Vec<HashSet<String>>,
    response: String,
}
