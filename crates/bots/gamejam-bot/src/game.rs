use super::*;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum GameType {
    Queued,
    Current,
    Skipped,
    Played,
}

#[derive(Serialize, Deserialize, Clone)]
struct GameSerialized {
    author: String,
    link: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(from = "GameSerialized", into = "GameSerialized")]
pub struct Game {
    pub author: String,
    pub link: String,
    pub name: Option<String>,
}

impl Game {
    pub fn new(author: String, link: String) -> Self {
        Self {
            author,
            name: Self::name_from_link(&link),
            link,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref().unwrap_or(&self.link)
    }

    pub fn to_string_name(&self, ping: bool) -> String {
        if ping {
            format!("{} from @{}", self.name(), self.author)
        } else {
            format!("{} from {}", self.name(), self.author)
        }
    }

    pub fn to_string_link(&self, ping: bool) -> String {
        if ping {
            format!("{} from @{}", self.link, self.author)
        } else {
            format!("{} from {}", self.link, self.author)
        }
    }

    fn name_from_link(link: &str) -> Option<String> {
        // Ludumdare
        let ludumdare = "https://ldjam.com/events/ludum-dare/";
        if link.starts_with(ludumdare) {
            let mut args = link[ludumdare.len()..].split('/');
            let _ld_number = args.next()?;
            let game_name = args.next()?;
            Some(game_name.to_owned())
        } else {
            None
        }
    }
}

impl From<GameSerialized> for Game {
    fn from(game: GameSerialized) -> Self {
        Self::new(game.author, game.link)
    }
}

impl From<Game> for GameSerialized {
    fn from(game: Game) -> Self {
        Self {
            author: game.author,
            link: game.link,
        }
    }
}
