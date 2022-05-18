use rocket::{
    response::stream::{Event, EventStream},
    serde::json::Json,
    tokio::time::{self, Duration},
    State,
};

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

#[get("/events")]
pub fn events(bot: &BotState) -> EventStream![Event + '_] {
    EventStream! {
        let mut interval = time::interval(Duration::from_secs(1));
        loop{
            let bot = bot.lock().await;
            let state = bot.serialize().collect::<Vec<_>>();
            yield Event::json(&state);
            interval.tick().await;
        }
    }
}
