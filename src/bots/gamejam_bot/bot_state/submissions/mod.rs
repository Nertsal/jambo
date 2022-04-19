use super::*;

mod submission;

pub use submission::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Submissions {
    #[serde(flatten)]
    pub queue: GamesQueue,
    #[serde(skip)]
    pub played_games: Vec<Submission>,
    pub skipped: Vec<Submission>,
}

impl Submissions {
    pub fn remove_game(
        &mut self,
        predicate: impl Fn(&Submission) -> bool + Copy,
    ) -> Option<Submission> {
        // Look in the queue
        self.queue.remove_game(predicate).or_else(|| {
            // Look in the skipped list
            self.skipped
                .iter()
                .enumerate()
                .find(|&(_, game)| predicate(game))
                .map(|(pos, _)| pos)
                .map(|pos| self.skipped.remove(pos))
        })
    }

    pub fn find_game(
        &self,
        predicate: impl Fn(&Submission) -> bool,
    ) -> Option<(&Submission, GameType)> {
        // Look in the queue
        let game = self.queue.get_queue().find(|game| predicate(game));
        if let Some(game) = game {
            return Some((game, GameType::Queued));
        }

        // Look in the skipped list
        let game = self.skipped.iter().find(|game| predicate(game));
        if let Some(game) = game {
            return Some((game, GameType::Skipped));
        }

        // Look in the played list
        let game = self.played_games.iter().find(|game| predicate(game));
        if let Some(game) = game {
            return Some((game, GameType::Played));
        }

        None
    }

    pub fn find_game_mut(
        &mut self,
        predicate: impl Fn(&Submission) -> bool,
    ) -> Option<(&mut Submission, GameType)> {
        // Look in the queue
        let game = self.queue.get_queue_mut().find(|game| predicate(game));
        if let Some(game) = game {
            return Some((game, GameType::Queued));
        }

        // Look in the skipped list
        let game = self.skipped.iter_mut().find(|game| predicate(game));
        if let Some(game) = game {
            return Some((game, GameType::Skipped));
        }

        // Look in the played list
        let game = self.played_games.iter_mut().find(|game| predicate(game));
        if let Some(game) = game {
            return Some((game, GameType::Played));
        }

        None
    }
}
