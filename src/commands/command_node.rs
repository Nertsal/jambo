use std::sync::Arc;

use super::*;

pub enum CommandNode<T> {
    Literal {
        literals: Vec<String>,
        child_nodes: Vec<CommandNode<T>>,
    },
    Argument {
        argument_type: ArgumentType,
        child_nodes: Vec<CommandNode<T>>,
    },
    ArgumentChoice {
        choices: Vec<String>,
        child_nodes: Vec<CommandNode<T>>,
    },
    Final {
        authority_level: AuthorityLevel,
        command: Command<T>,
    },
}

pub type Argument = String;

pub type SenderName = String;

pub type Response = Option<String>;

pub type Command<T> = Arc<dyn Fn(&mut T, SenderName, Vec<Argument>) -> Response + Send + Sync>;

#[derive(Clone, Copy)]
pub enum ArgumentType {
    Word,
    Line,
}

macro_rules! child_nodes {
    ( $child_nodes: expr, $message: expr, $arguments: expr ) => {
        for child_node in $child_nodes {
            if let Some((final_node, arguments)) =
                child_node.check_node($message, $arguments.clone())
            {
                return Some((final_node, arguments));
            }
        }
    };
}

impl<T> CommandNode<T> {
    pub fn check_node(
        &self,
        message: &str,
        mut arguments: Vec<Argument>,
    ) -> Option<(&CommandNode<T>, Vec<Argument>)> {
        match self {
            CommandNode::Literal {
                literals,
                child_nodes,
            } => literals
                .iter()
                .find(|&literal| message.starts_with(literal))
                .map(|literal| {
                    let message = message[literal.len()..].trim();
                    child_nodes!(child_nodes, message, arguments);
                    None
                })
                .flatten(),

            CommandNode::Argument {
                argument_type,
                child_nodes,
            } => match argument_type {
                ArgumentType::Word => message.split_whitespace().next(),
                ArgumentType::Line => {
                    if message.trim().is_empty() {
                        None
                    } else {
                        Some(message)
                    }
                }
            }
            .map(|argument| {
                let message = message[argument.len()..].trim();
                arguments.push(argument.to_owned());
                child_nodes!(child_nodes, message, arguments);
                None
            })
            .flatten(),

            CommandNode::ArgumentChoice {
                choices,
                child_nodes,
            } => message
                .split_whitespace()
                .next()
                .map(|argument| {
                    choices
                        .iter()
                        .find(|&choice| choice == argument)
                        .map(|choice| {
                            arguments.push(choice.to_owned());
                            child_nodes!(child_nodes, message, arguments);
                            None
                        })
                        .flatten()
                })
                .flatten(),

            CommandNode::Final { .. } => {
                if message.is_empty() {
                    Some((self, arguments))
                } else {
                    None
                }
            }
        }
    }
}
