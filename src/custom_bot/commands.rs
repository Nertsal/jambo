use super::*;

impl CommandBot<Self> for CustomBot {
    fn get_commands(&self) -> &BotCommands<Self> {
        &self.commands
    }
}

impl CustomBot {
    pub fn commands() -> BotCommands<Self> {
        BotCommands {
            commands: vec![CommandNode::LiteralNode {
                literal: "command".to_owned(),
                child_nodes: vec![
                    CommandNode::LiteralNode {
                        literal: "new".to_owned(),
                        child_nodes: vec![CommandNode::ArgumentNode {
                            argument_type: ArgumentType::Word,
                            child_node: Box::new(CommandNode::ArgumentNode {
                                argument_type: ArgumentType::Line,
                                child_node: Box::new(CommandNode::FinalNode {
                                    authority_level: AuthorityLevel::Moderator,
                                    command: |bot, _, args| {
                                        if let [command_name, command_response] = args.as_slice() {
                                            let response = Some(format!(
                                                "Added new command !{}: {}",
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
                                    },
                                }),
                            }),
                        }],
                    },
                    CommandNode::LiteralNode {
                        literal: "remove".to_owned(),
                        child_nodes: vec![CommandNode::ArgumentNode {
                            argument_type: ArgumentType::Word,
                            child_node: Box::new(CommandNode::FinalNode {
                                authority_level: AuthorityLevel::Moderator,
                                command: |bot, _, mut args| {
                                    let command_name = args.remove(0);
                                    if let Some(command_response) =
                                        bot.config.commands.remove(&command_name)
                                    {
                                        let response = Some(format!(
                                            "Removed command {}: {}",
                                            command_name, command_response
                                        ));
                                        let com_index = bot
                                            .commands
                                            .commands
                                            .iter()
                                            .position(|command| match command {
                                                CommandNode::LiteralNode { literal, .. } => {
                                                    *literal == command_name
                                                }
                                                _ => false,
                                            })
                                            .unwrap();
                                        bot.commands.commands.remove(com_index);
                                        bot.config.save().unwrap();
                                        return response;
                                    }
                                    None
                                },
                            }),
                        }],
                    },
                    CommandNode::LiteralNode {
                        literal: "edit".to_owned(),
                        child_nodes: vec![CommandNode::ArgumentNode {
                            argument_type: ArgumentType::Word,
                            child_node: Box::new(CommandNode::ArgumentNode {
                                argument_type: ArgumentType::Line,
                                child_node: Box::new(CommandNode::FinalNode {
                                    authority_level: AuthorityLevel::Moderator,
                                    command: |bot, _, args| {
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
                                    },
                                }),
                            }),
                        }],
                    },
                ],
            }],
        }
    }
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
        self.commands.commands.push(CommandNode::LiteralNode {
            literal: command_name,
            child_nodes: vec![CommandNode::FinalNode {
                authority_level: AuthorityLevel::Any,
                command: |bot, _, _| {
                    unimplemented!()
                    // Some(bot.config.commands[&command_name].clone())
                },
            }],
        });
    }
}
