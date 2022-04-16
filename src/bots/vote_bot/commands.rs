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
        Commands {
            commands: vec![CommandNode::Literal {
                literals: vec!["!vote".to_owned()],
                child_nodes: vec![
                    CommandNode::Literal {
                        literals: vec!["start".to_owned()],
                        child_nodes: vec![CommandNode::final_node(
                            true,
                            AuthorityLevel::Broadcaster as usize,
                            Arc::new(|bot, _, _| bot.vote_start()),
                        )],
                    },
                    CommandNode::Literal {
                        literals: vec!["finish".to_owned()],
                        child_nodes: vec![CommandNode::final_node(
                            true,
                            AuthorityLevel::Broadcaster as usize,
                            Arc::new(|bot, _, _| bot.vote_finish()),
                        )],
                    },
                    CommandNode::Argument {
                        argument_type: ArgumentType::Line,
                        child_nodes: vec![CommandNode::final_node(
                            true,
                            AuthorityLevel::Viewer as usize,
                            Arc::new(|bot, sender, mut args| {
                                let vote = args.remove(0);
                                bot.vote(sender.name.clone(), vote)
                            }),
                        )],
                    },
                ],
            }],
        }
    }
}
