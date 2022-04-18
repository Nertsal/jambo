use rocket::{serde::json::Json, State};

use super::*;

type BotState = State<Arc<MutexBot>>;

#[get("/")]
pub fn index() -> &'static str {
    "This is a twitch bot made by Nertsal (https://github.com/Nertsal/jambo)\n"
}

#[get("/state")]
pub async fn get_state(bot: &BotState) -> Json<Vec<SerializedBot>> {
    let bot = bot.lock().await;
    Json(bot.serialize().collect())
}
