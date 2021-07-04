use std::sync::Arc;

use super::*;

impl CommandBot<Self> for VoteBot {
    fn get_commands(&self) -> &BotCommands<Self> {
        &self.commands
    }
}

impl VoteBot {
    pub fn vote_start(&mut self) -> Option<String> {
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

    pub fn vote_finish(&mut self) -> Option<String> {
        let vote_mode = std::mem::replace(&mut self.vote_mode, VoteMode::Inactive);
        match vote_mode {
            VoteMode::Active { votes } => {
                self.update_status(&serde_json::to_string(&votes).unwrap());
                Some(format!(
                    "The voting has finished with the total of {} votes.",
                    votes.len()
                ))
            }
            VoteMode::Inactive => Some(format!("The voting should be started first: !vote start")),
        }
    }

    pub fn vote(&mut self, voter: String, vote: String) -> Option<String> {
        match &mut self.vote_mode {
            VoteMode::Active { votes } => {
                votes.insert(voter, vote);
            }
            _ => (),
        }
        None
    }

    pub fn commands() -> BotCommands<Self> {
        BotCommands {
            commands: vec![CommandNode::LiteralNode {
                literals: vec!["!vote".to_owned()],
                child_nodes: vec![
                    CommandNode::LiteralNode {
                        literals: vec!["start".to_owned()],
                        child_nodes: vec![CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Broadcaster,
                            command: Arc::new(|bot, _, _| bot.vote_start()),
                        }],
                    },
                    CommandNode::LiteralNode {
                        literals: vec!["finish".to_owned()],
                        child_nodes: vec![CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Broadcaster,
                            command: Arc::new(|bot, _, _| bot.vote_finish()),
                        }],
                    },
                    CommandNode::ArgumentNode {
                        argument_type: ArgumentType::Line,
                        child_node: Box::new(CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Any,
                            command: Arc::new(|bot, sender_name, mut args| {
                                let vote = args.remove(0);
                                bot.vote(sender_name, vote)
                            }),
                        }),
                    },
                ],
            }],
        }
    }
}
