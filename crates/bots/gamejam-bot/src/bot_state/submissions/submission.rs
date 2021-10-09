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
    authors: Vec<String>,
    link: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(from = "GameSerialized", into = "GameSerialized")]
pub struct Submission {
    pub authors: Vec<String>,
    pub link: String,
    pub name: Option<String>,
}

impl Submission {
    pub fn new(authors: Vec<String>, link: String) -> Self {
        Self {
            authors,
            name: Self::name_from_link(&link),
            link,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref().unwrap_or(&self.link)
    }

    pub fn to_string_name(&self, ping: bool) -> String {
        if ping {
            format!("{} from @{}", self.name(), self.authors[0])
        } else {
            format!("{} from {}", self.name(), self.authors[0])
        }
    }

    pub fn to_string_link(&self, ping: bool) -> String {
        if ping {
            format!("{} from @{}", self.link, self.authors[0])
        } else {
            format!("{} from {}", self.link, self.authors[0])
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

impl From<GameSerialized> for Submission {
    fn from(game: GameSerialized) -> Self {
        Self::new(game.authors, game.link)
    }
}

impl From<Submission> for GameSerialized {
    fn from(game: Submission) -> Self {
        Self {
            authors: game.authors,
            link: game.link,
        }
    }
}
