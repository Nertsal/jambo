use super::*;

pub struct CommandCompleter {
    pub completion_tree: Vec<CompletionNode>,
}

impl<Term: linefeed::Terminal> linefeed::Completer<Term> for CommandCompleter {
    fn complete(
        &self,
        _word: &str,
        prompter: &linefeed::Prompter<Term>,
        _start: usize,
        _end: usize,
    ) -> Option<Vec<linefeed::Completion>> {
        let message = prompter.buffer();
        Some(
            self.completion_tree
                .iter()
                .flat_map(|node| node.complete(message))
                .collect(),
        )
    }
}

pub enum CompletionNode {
    Literal {
        literals: Vec<String>,
        child_nodes: Vec<CompletionNode>,
    },
    Argument {
        argument_type: ArgumentType,
        child_nodes: Vec<CompletionNode>,
    },
    Final,
}

impl CompletionNode {
    fn complete(&self, message: &str) -> Vec<linefeed::Completion> {
        let mut completions = Vec::new();
        match self {
            CompletionNode::Literal {
                literals,
                child_nodes,
            } => {
                for literal in literals {
                    if literal.starts_with(message) && literal != message {
                        completions.push(linefeed::Completion::simple(literal.clone()));
                    }
                }
                if let Some(literal) = literals
                    .iter()
                    .find(|&literal| message.starts_with(literal))
                {
                    let message = message[literal.len()..].trim();
                    for child_node in child_nodes {
                        completions.append(&mut child_node.complete(message));
                    }
                }
            }
            CompletionNode::Argument {
                argument_type,
                child_nodes,
            } => match argument_type {
                ArgumentType::Word => {
                    if let Some(argument) = message.split_whitespace().next() {
                        let message = message[argument.len()..].trim();
                        for child_node in child_nodes {
                            completions.append(&mut child_node.complete(message));
                        }
                    }
                }
                ArgumentType::Line => (),
            },
            CompletionNode::Final => (),
        }
        completions
    }
}

impl<T> From<&CommandNode<T>> for CompletionNode {
    fn from(node: &CommandNode<T>) -> Self {
        match node {
            CommandNode::ArgumentNode {
                argument_type,
                child_nodes,
            } => CompletionNode::Argument {
                argument_type: *argument_type,
                child_nodes: commands_to_completion(child_nodes),
            },
            CommandNode::LiteralNode {
                literals,
                child_nodes,
            } => CompletionNode::Literal {
                literals: literals.clone(),
                child_nodes: commands_to_completion(child_nodes),
            },
            CommandNode::FinalNode { .. } => CompletionNode::Final,
        }
    }
}

pub fn commands_to_completion<T>(commands: &Vec<CommandNode<T>>) -> Vec<CompletionNode> {
    commands
        .iter()
        .map(|node| CompletionNode::from(node))
        .collect()
}
