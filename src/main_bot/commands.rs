use super::*;

impl MainBot {
    pub fn commands() -> Commands<Self> {
        Commands::new(vec![CommandNode::literal(
            ["test"],
            vec![CommandNode::final_node(
                true,
                AuthorityLevel::Viewer as _,
                Arc::new(|_, sender, args| {
                    Some(format!("Got a message from {sender:?}: {args:?}"))
                }),
            )],
        )])
    }
}
