use super::*;

impl MainBot {
    fn enable(&mut self, bot_name: &str) -> Response {
        if self.bots.active.contains_key(bot_name) {
            return Some(format!("{bot_name} is already active"));
        }
        match self.bots.constructors.get(bot_name) {
            Some(constructor) => {
                let bot = constructor(&self.cli);
                self.bots.active.insert(bot_name.to_owned(), bot);
                Some(format!("{bot_name} is now active"))
            }
            None => Some(format!("I don't know about {bot_name}")),
        }
    }

    fn disable(&mut self, bot_name: &str) -> Response {
        match self.bots.active.remove(bot_name) {
            Some(_) => Some(format!("{bot_name} is now resting")),
            None => {
                if self.bots.constructors.contains_key(bot_name) {
                    Some(format!("{bot_name} is already off"))
                } else {
                    Some(format!("I don't know about {bot_name}"))
                }
            }
        }
    }

    pub fn commands() -> Commands<Self> {
        Commands::new(vec![
            CommandNode::literal(
                ["!enable"],
                vec![CommandNode::argument(
                    ArgumentType::Word,
                    vec![CommandNode::final_node(
                        true,
                        AuthorityLevel::Viewer as _,
                        Arc::new(|bot, _, args| {
                            args.first().and_then(|bot_name| bot.enable(bot_name))
                        }),
                    )],
                )],
            ),
            CommandNode::literal(
                ["!disable"],
                vec![CommandNode::argument(
                    ArgumentType::Word,
                    vec![CommandNode::final_node(
                        true,
                        AuthorityLevel::Viewer as _,
                        Arc::new(|bot, _, args| {
                            args.first().and_then(|bot_name| bot.disable(bot_name))
                        }),
                    )],
                )],
            ),
            CommandNode::literal(
                ["!shutdown"],
                vec![CommandNode::final_node(
                    true,
                    AuthorityLevel::Broadcaster as _,
                    Arc::new(|bot, _, _| {
                        bot.queue_shutdown = true;
                        Some(format!("Shutting down..."))
                    })
                )]
            )
        ])
    }
}
