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

    fn reset(&mut self, bot_name: &str) -> Response {
        self.disable(bot_name);
        self.enable(bot_name)
    }

    fn reset_all(&mut self) -> Response {
        let active = self
            .bots
            .active
            .drain()
            .map(|(name, _)| name)
            .collect::<Vec<_>>();
        let mut res = String::new();
        for bot_name in active {
            if let Some(ans) = self.enable(&bot_name) {
                res += ans.as_str();
                res += ". ";
            }
        }
        Some(res)
    }

    pub fn commands() -> Commands<Self> {
        let reset = CommandBuilder::<Self, _>::new().word().finalize(
            true,
            AuthorityLevel::Moderator as _,
            Arc::new(|bot, _, args| bot.reset(&args[0])),
        );

        let reset_all = CommandBuilder::<Self, _>::new().literal(["all"]).finalize(
            true,
            AuthorityLevel::Moderator as _,
            Arc::new(|bot, _, _| bot.reset_all()),
        );

        Commands::new(vec![
            CommandBuilder::new().literal(["!enable"]).word().finalize(
                true,
                AuthorityLevel::Moderator as _,
                Arc::new(|bot, _, args| bot.enable(&args[0])),
            ),
            CommandBuilder::new().literal(["!disable"]).word().finalize(
                true,
                AuthorityLevel::Moderator as _,
                Arc::new(|bot, _, args| bot.disable(&args[0])),
            ),
            CommandBuilder::new().literal(["!shutdown"]).finalize(
                true,
                AuthorityLevel::Broadcaster as _,
                Arc::new(|bot, _, _| {
                    bot.queue_shutdown = true;
                    Some(format!("Shutting down..."))
                }),
            ),
            CommandBuilder::new()
                .literal(["!reset"])
                .split([reset, reset_all]),
        ])
    }
}
