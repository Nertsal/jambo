use super::*;

impl VoteBot {
    pub fn vote_start(&mut self) -> Response {
        match &self.vote_mode {
            VoteMode::Active { .. } => Some(format!("The voting is in progress.")),
            VoteMode::Inactive => {
                self.vote_mode = VoteMode::Active {
                    votes: HashMap::new(),
                };
                self.update_status("The voting is in progress");
                Some(format!("The voting has started. Type !vote <your vote>"))
            }
        }
    }

    pub fn vote_finish(&mut self) -> Response {
        let vote_mode = std::mem::replace(&mut self.vote_mode, VoteMode::Inactive);
        match vote_mode {
            VoteMode::Active { votes } => {
                let voters = votes.len();
                let votes_count = {
                    let mut votes_count = HashMap::new();
                    for (_, vote) in votes {
                        *votes_count.entry(vote).or_insert(0) += 1;
                    }
                    let mut votes_count: Vec<(String, usize)> = votes_count.into_iter().collect();
                    votes_count.sort_by(|(vote_a, _), (vote_b, _)| vote_a.cmp(vote_b));
                    votes_count
                };
                self.update_status(&serde_json::to_string(&votes_count).unwrap());
                Some(format!(
                    "The voting has finished with the total of {} votes and {} unique ones.",
                    voters,
                    votes_count.len(),
                ))
            }
            VoteMode::Inactive => Some(format!("The voting should be started first: !vote start")),
        }
    }

    pub fn vote(&mut self, voter: String, vote: String) -> Response {
        match &mut self.vote_mode {
            VoteMode::Active { votes } => {
                votes.insert(voter, vote.to_lowercase());
            }
            _ => (),
        }
        None
    }

    pub fn commands() -> Commands<Self> {
        let start = CommandBuilder::<Self, _>::new()
            .literal(["start"])
            .finalize(
                true,
                AuthorityLevel::Broadcaster as _,
                Arc::new(|bot, _, _| bot.vote_start()),
            );

        let finish = CommandBuilder::<Self, _>::new()
            .literal(["finish"])
            .finalize(
                true,
                AuthorityLevel::Broadcaster as _,
                Arc::new(|bot, _, _| bot.vote_finish()),
            );

        let vote = CommandBuilder::<Self, Sender>::new().line().finalize(
            true,
            AuthorityLevel::Broadcaster as _,
            Arc::new(|bot, sender, args| bot.vote(sender.name.to_owned(), args[0].to_owned())),
        );

        Commands {
            commands: vec![CommandBuilder::new()
                .literal(["!vote"])
                .split([start, finish, vote])],
        }
    }
}
