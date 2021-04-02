use std::sync::Arc;

use super::*;

impl CommandBot<Self> for QuoteBot {
    fn get_commands(&self) -> &BotCommands<Self> {
        &self.commands
    }
}

impl QuoteBot {
    pub fn commands() -> BotCommands<Self> {
        BotCommands {
            commands: vec![CommandNode::LiteralNode {
                literal: "quote".to_owned(),
                child_nodes: vec![
                    CommandNode::LiteralNode {
                        literal: "add".to_owned(),
                        child_nodes: vec![CommandNode::ArgumentNode {
                            argument_type: ArgumentType::Line,
                            child_node: Box::new(CommandNode::FinalNode {
                                authority_level: AuthorityLevel::Moderator,
                                command: Arc::new(|bot, _, mut args| {
                                    let quote = args.remove(0);
                                    let quote_id = bot.config.id_generator.gen();
                                    let response = Some(format!(
                                        "Added new quote {}: {}",
                                        quote_id.raw(),
                                        quote
                                    ));
                                    bot.config.quotes.insert(quote_id, quote);
                                    bot.config.save().unwrap();
                                    response
                                }),
                            }),
                        }],
                    },
                    CommandNode::LiteralNode {
                        literal: "delete".to_owned(),
                        child_nodes: vec![CommandNode::ArgumentNode {
                            argument_type: ArgumentType::Word,
                            child_node: Box::new(CommandNode::FinalNode {
                                authority_level: AuthorityLevel::Moderator,
                                command: Arc::new(|bot, _, mut args| {
                                    let quote_id = args.remove(0);
                                    if let Ok(quote_id) = serde_json::from_str(quote_id.as_str()) {
                                        if let Some(quote) = bot.config.quotes.remove(&quote_id) {
                                            let response = Some(format!(
                                                "Deleted quote {:?}: {}",
                                                quote_id.raw(),
                                                quote
                                            ));
                                            bot.config.save().unwrap();
                                            return response;
                                        }
                                    }
                                    None
                                }),
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
                                    command: Arc::new(|bot, _, args| {
                                        if let [quote_id, quote] = args.as_slice() {
                                            if let Ok(quote_id) = serde_json::from_str(quote_id) {
                                                let response = if let Some(old_quote) =
                                                    bot.config.quotes.get_mut(&quote_id)
                                                {
                                                    let response = Some(format!(
                                                        "Edited quote {}: {}. New quote: {}",
                                                        quote_id.raw(),
                                                        old_quote,
                                                        quote
                                                    ));
                                                    *old_quote = quote.to_owned();
                                                    response
                                                } else {
                                                    let response = Some(format!(
                                                        "Added new quote {}: {}",
                                                        quote_id.raw(),
                                                        quote
                                                    ));
                                                    bot.config
                                                        .quotes
                                                        .insert(quote_id, quote.to_owned());
                                                    response
                                                };
                                                bot.config.save().unwrap();
                                                return response;
                                            }
                                        }

                                        None
                                    }),
                                }),
                            }),
                        }],
                    },
                    CommandNode::ArgumentNode {
                        argument_type: ArgumentType::Word,
                        child_node: Box::new(CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Any,
                            command: Arc::new(|bot, _, mut args| {
                                let quote_id = args.remove(0);
                                if let Ok(quote_id) = serde_json::from_str(quote_id.as_str()) {
                                    if let Some(quote) = bot.config.quotes.get(&quote_id) {
                                        let response = Some(quote.clone());
                                        return response;
                                    }
                                }
                                None
                            }),
                        }),
                    },
                ],
            }],
        }
    }
}
