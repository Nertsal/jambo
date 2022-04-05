use super::*;

#[derive(Serialize, Deserialize, Default)]
pub struct GamesQueue {
    returned_queue: VecDeque<Submission>,
    games_queue: VecDeque<Submission>,
}

impl GamesQueue {
    pub fn get_queue(&self) -> impl Iterator<Item = &Submission> {
        self.returned_queue.iter().chain(self.games_queue.iter())
    }

    pub fn get_queue_mut(&mut self) -> impl Iterator<Item = &mut Submission> {
        self.returned_queue
            .iter_mut()
            .chain(self.games_queue.iter_mut())
    }

    pub fn queue_game(&mut self, game: Submission) {
        self.games_queue.push_back(game);
    }

    pub fn return_game(&mut self, game: Submission) {
        self.returned_queue.push_back(game);
    }

    pub fn return_game_front(&mut self, game: Submission) {
        self.returned_queue.push_front(game);
    }

    pub fn next(&mut self) -> Option<Submission> {
        self.returned_queue
            .pop_front()
            .or_else(|| self.games_queue.pop_front())
    }

    pub fn drain_all<'a>(&'a mut self) -> impl Iterator<Item = Submission> + 'a {
        self.returned_queue
            .drain(..)
            .chain(self.games_queue.drain(..))
    }

    pub fn remove_game(&mut self, predicate: impl Fn(&Submission) -> bool) -> Option<Submission> {
        let pos = self
            .returned_queue
            .iter()
            .enumerate()
            .find(|&(_, game)| predicate(game))
            .map(|(pos, _)| pos);
        if let Some(pos) = pos {
            return self.returned_queue.remove(pos);
        }

        let pos = self
            .games_queue
            .iter()
            .enumerate()
            .find(|&(_, game)| predicate(game))
            .map(|(pos, _)| pos);
        if let Some(pos) = pos {
            return self.games_queue.remove(pos);
        }

        None
    }
}
