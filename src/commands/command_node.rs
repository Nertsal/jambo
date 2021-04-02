use std::sync::Arc;

use super::*;

pub enum CommandNode<T> {
    ArgumentNode {
        argument_type: ArgumentType,
        child_node: Box<CommandNode<T>>,
    },
    LiteralNode {
        literal: String,
        child_nodes: Vec<CommandNode<T>>,
    },
    FinalNode {
        authority_level: AuthorityLevel,
        command: Command<T>,
    },
}

pub type Argument = String;

pub type SenderName = String;

pub type Command<T> =
    Arc<dyn Fn(&mut T, SenderName, Vec<Argument>) -> Option<String> + Send + Sync>;

pub enum ArgumentType {
    Word,
    Line,
}

impl<T> CommandNode<T> {
    pub fn check_node(
        &self,
        message: &str,
        mut arguments: Vec<Argument>,
    ) -> Option<(&CommandNode<T>, Vec<Argument>)> {
        match self {
            CommandNode::ArgumentNode {
                argument_type,
                child_node,
            } => {
                if let Some(argument) = match argument_type {
                    ArgumentType::Word => message.split_whitespace().next(),
                    ArgumentType::Line => Some(message),
                } {
                    let message = message[argument.len()..].trim();
                    arguments.push(argument.to_owned());
                    child_node.check_node(message, arguments)
                } else {
                    None
                }
            }
            CommandNode::LiteralNode {
                literal,
                child_nodes,
            } => {
                if message.starts_with(literal) {
                    let message = message[literal.len()..].trim();
                    for child_node in child_nodes {
                        if let Some((final_node, arguments)) =
                            child_node.check_node(message, arguments.clone())
                        {
                            return Some((final_node, arguments));
                        }
                    }
                    None
                } else {
                    None
                }
            }
            CommandNode::FinalNode { .. } => {
                if message.is_empty() {
                    Some((self, arguments))
                } else {
                    None
                }
            }
        }
    }
}
