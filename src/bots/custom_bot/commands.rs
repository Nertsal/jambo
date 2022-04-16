use super::*;

impl CustomBot {
    fn new_command(&mut self, command_name: String, command_response: String) -> bool {
        if self.config.commands.contains_key(&command_name) {
            false
        } else {
            self.update_command(command_name, command_response);
            true
        }
    }

    fn update_command(&mut self, command_name: String, command_response: String) {
        self.config
            .commands
            .insert(command_name.clone(), command_response.clone());
        self.push_command(command_name);
        self.config.save().unwrap();
    }

    pub fn push_command(&mut self, command_name: String) {
        self.commands.commands.push(CommandNode::Literal {
            literals: vec![command_name.clone()],
            child_nodes: vec![CommandNode::final_node(
                true,
                AuthorityLevel::Viewer as usize,
                Arc::new(move |bot, _, _| Some(bot.config.commands[&command_name].clone())),
            )],
        });
    }

    pub fn commands() -> Commands<Self> {
        Commands {
            commands: vec![CommandNode::Literal {
                literals: vec!["!command".to_owned()],
                child_nodes: vec![
                    CommandNode::Literal {
                        literals: vec!["new".to_owned()],
                        child_nodes: vec![CommandNode::Argument {
                            argument_type: ArgumentType::Word,
                            child_nodes: vec![CommandNode::Argument {
                                argument_type: ArgumentType::Line,
                                child_nodes: vec![CommandNode::final_node(
                                    true,
                                    AuthorityLevel::Moderator as usize,
                                    Arc::new(|bot, _, args| {
                                        if let [command_name, command_response] = args.as_slice() {
                                            let response = Some(format!(
                                                "Added new command {}: {}",
                                                command_name, command_response
                                            ));
                                            if bot.new_command(
                                                command_name.to_owned(),
                                                command_response.to_owned(),
                                            ) {
                                                return response;
                                            }
                                        }
                                        None
                                    }),
                                )],
                            }],
                        }],
                    },
                    CommandNode::Literal {
                        literals: vec!["delete".to_owned()],
                        child_nodes: vec![CommandNode::Argument {
                            argument_type: ArgumentType::Word,
                            child_nodes: vec![CommandNode::final_node(
                                true,
                                AuthorityLevel::Moderator as usize,
                                Arc::new(|bot, _, mut args| {
                                    let command_name = args.remove(0);
                                    if let Some(command_response) =
                                        bot.config.commands.remove(&command_name)
                                    {
                                        let response = Some(format!(
                                            "Deleted command {}: {}",
                                            command_name, command_response
                                        ));
                                        let com_index = bot
                                            .commands
                                            .commands
                                            .iter()
                                            .position(|command| match command {
                                                CommandNode::Literal { literals, .. } => {
                                                    literals.contains(&command_name)
                                                }
                                                _ => false,
                                            })
                                            .unwrap();
                                        bot.commands.commands.remove(com_index);
                                        bot.config.save().unwrap();
                                        return response;
                                    }
                                    None
                                }),
                            )],
                        }],
                    },
                    CommandNode::Literal {
                        literals: vec!["edit".to_owned()],
                        child_nodes: vec![CommandNode::Argument {
                            argument_type: ArgumentType::Word,
                            child_nodes: vec![CommandNode::Argument {
                                argument_type: ArgumentType::Line,
                                child_nodes: vec![CommandNode::final_node(
                                    true,
                                    AuthorityLevel::Moderator as usize,
                                    Arc::new(|bot, _, args| {
                                        if let [command_name, command_response] = args.as_slice() {
                                            if let Some(old_response) =
                                                bot.config.commands.get_mut(command_name)
                                            {
                                                let response = Some(format!(
                                                    "Edited command {}: {}. New command: {}",
                                                    command_name, old_response, command_response
                                                ));
                                                bot.update_command(
                                                    command_name.to_owned(),
                                                    command_response.to_owned(),
                                                );
                                                return response;
                                            }
                                        }
                                        None
                                    }),
                                )],
                            }],
                        }],
                    },
                ],
            }],
        }
    }
}