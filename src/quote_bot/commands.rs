use rand::seq::SliceRandom;
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
                literals: vec!["!quote".to_owned()],
                child_nodes: vec![
                    CommandNode::FinalNode {
                        authority_level: AuthorityLevel::Viewer,
                        command: Arc::new(|bot, _, _| {
                            if let Some(random_quote_name) = bot
                                .config
                                .quotes
                                .keys()
                                .collect::<Vec<&String>>()
                                .choose(&mut rand::thread_rng())
                            {
                                Some(format!(
                                    "Quote {}: {}",
                                    random_quote_name, bot.config.quotes[random_quote_name as &str]
                                ))
                            } else {
                                Some(format!(
                                    "No quotes yet. Add new ones with !quote add <quote>"
                                ))
                            }
                        }),
                    },
                    CommandNode::LiteralNode {
                        literals: vec!["add".to_owned()],
                        child_nodes: vec![CommandNode::ArgumentNode {
                            argument_type: ArgumentType::Word,
                            child_nodes: vec!(CommandNode::ArgumentNode {
                                argument_type: ArgumentType::Line,
                                child_nodes: vec!(CommandNode::FinalNode {
                                    authority_level: AuthorityLevel::Moderator,
                                    command: Arc::new(|bot, _, args| {
                                        if let [quote_name, quote] = args.as_slice() {
                                            let response =
                                                if bot.config.quotes.contains_key(quote_name) {
                                                    Some(format!(
                                                        "A quote with the name {} already exists",
                                                        quote_name
                                                    ))
                                                } else {
                                                    let response = Some(format!(
                                                        "Added new quote {}: {}",
                                                        quote_name, quote
                                                    ));
                                                    bot.config.quotes.insert(
                                                        quote_name.to_owned(),
                                                        quote.to_owned(),
                                                    );
                                                    bot.config.save().unwrap();
                                                    response
                                                };
                                            return response;
                                        }
                                        None
                                    }),
                                }),
                            }),
                        }],
                    },
                    CommandNode::LiteralNode {
                        literals: vec!["delete".to_owned()],
                        child_nodes: vec![CommandNode::ArgumentNode {
                            argument_type: ArgumentType::Word,
                            child_nodes: vec!(CommandNode::FinalNode {
                                authority_level: AuthorityLevel::Moderator,
                                command: Arc::new(|bot, _, mut args| {
                                    let quote_name = args.remove(0);
                                    if let Some(quote) = bot.config.quotes.remove(&quote_name) {
                                        let response = Some(format!(
                                            "Deleted quote {:?}: {}",
                                            quote_name, quote
                                        ));
                                        bot.config.save().unwrap();
                                        return response;
                                    }
                                    None
                                }),
                            }),
                        }],
                    },
                    CommandNode::LiteralNode {
                        literals: vec!["edit".to_owned()],
                        child_nodes: vec![CommandNode::ArgumentNode {
                            argument_type: ArgumentType::Word,
                            child_nodes: vec!(CommandNode::ArgumentNode {
                                argument_type: ArgumentType::Line,
                                child_nodes: vec!(CommandNode::FinalNode {
                                    authority_level: AuthorityLevel::Moderator,
                                    command: Arc::new(|bot, _, args| {
                                        if let [quote_name, quote] = args.as_slice() {
                                            let response = if let Some(old_quote) =
                                                bot.config.quotes.get_mut(quote_name)
                                            {
                                                let response = Some(format!(
                                                    "Edited quote {}: {}. New quote: {}",
                                                    quote_name, old_quote, quote
                                                ));
                                                *old_quote = quote.to_owned();
                                                response
                                            } else {
                                                let response = Some(format!(
                                                    "Added new quote {}: {}",
                                                    quote_name, quote
                                                ));
                                                bot.config.quotes.insert(
                                                    quote_name.to_owned(),
                                                    quote.to_owned(),
                                                );
                                                response
                                            };
                                            bot.config.save().unwrap();
                                            return response;
                                        }
                                        None
                                    }),
                                }),
                            }),
                        }],
                    },
                    CommandNode::LiteralNode {
                        literals: vec!["rename".to_owned()],
                        child_nodes: vec![CommandNode::ArgumentNode {
                            argument_type: ArgumentType::Word,
                            child_nodes: vec!(CommandNode::ArgumentNode {
                                argument_type: ArgumentType::Word,
                                child_nodes: vec!(CommandNode::FinalNode {
                                    authority_level: AuthorityLevel::Moderator,
                                    command: Arc::new(|bot, _, args| {
                                        if let [quote_name, quote_new_name] = args.as_slice() {
                                            let response =
                                                if bot.config.quotes.contains_key(quote_new_name) {
                                                    Some(format!(
                                                        "A quote with name {} already exists",
                                                        quote_new_name
                                                    ))
                                                } else if let Some(quote) =
                                                    bot.config.quotes.remove(quote_name)
                                                {
                                                    let response = Some(format!(
                                                        "Changed quote's name from {} to {}",
                                                        quote_name, quote_new_name
                                                    ));
                                                    bot.config
                                                        .quotes
                                                        .insert(quote_new_name.to_owned(), quote);
                                                    response
                                                } else {
                                                    Some(format!(
                                                        "No quote with name {} found",
                                                        quote_name
                                                    ))
                                                };
                                            bot.config.save().unwrap();
                                            return response;
                                        }
                                        None
                                    }),
                                }),
                            }),
                        }],
                    },
                    CommandNode::ArgumentNode {
                        argument_type: ArgumentType::Word,
                        child_nodes: vec!(CommandNode::FinalNode {
                            authority_level: AuthorityLevel::Viewer,
                            command: Arc::new(|bot, _, mut args| {
                                let quote_name = args.remove(0);
                                if let Some(quote) = bot.config.quotes.get(&quote_name) {
                                    let response = Some(quote.clone());
                                    return response;
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
