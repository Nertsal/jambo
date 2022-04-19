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
                self.save_bots().expect("Failed to save state");
                Some(format!("{bot_name} is now active"))
            }
            None => Some(format!("I don't know about {bot_name}")),
        }
    }

    fn disable(&mut self, bot_name: &str) -> Response {
        match self.bots.active.remove(bot_name) {
            Some(_) => {
                self.save_bots().expect("Failed to save state");
                Some(format!("{bot_name} is now resting"))
            }
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

    fn backup_create(&self, backup_path: impl AsRef<std::path::Path>) -> std::io::Result<Response> {
        let path: &std::path::Path = "backups/".as_ref();
        let path = &path.join(backup_path.as_ref());
        clear_dir(path)?;
        copy_dir::copy_dir("config", path.join("config"))?;
        copy_dir::copy_dir("status", path.join("status"))?;
        Ok(Some(format!("Backup created")))
    }

    fn backup_load(
        &mut self,
        backup_path: impl AsRef<std::path::Path>,
    ) -> std::io::Result<Response> {
        // Backup current state
        self.backup_create("temp")?;
        // Try loading backup
        fn load(backup_path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
            let path: &std::path::Path = "backups/".as_ref();
            let path = &path.join(backup_path.as_ref());
            std::fs::remove_dir_all("config").unwrap_or(());
            copy_dir::copy_dir(path.join("config"), "config")?;
            std::fs::remove_dir_all("status").unwrap_or(());
            copy_dir::copy_dir(path.join("status"), "status")?;
            Ok(())
        }
        match load(backup_path) {
            Ok(_) => {
                self.reset_all();
                std::fs::remove_dir_all("backups/temp")?;
                Ok(Some(format!("Backup loaded")))
            }
            Err(err) => {
                self.log(LogType::Error, &format!("Failed to load backup: {err}"));
                load("temp")?;
                Ok(Some(format!("Failed to load backup")))
            }
        }
    }

    pub fn commands() -> Commands<Self> {
        let reset = CommandBuilder::<Self>::new().word().finalize(
            true,
            AuthorityLevel::Moderator as _,
            Arc::new(|bot, _, args| bot.reset(&args[0])),
        );

        let reset_all = CommandBuilder::<Self>::new().literal(["all"]).finalize(
            true,
            AuthorityLevel::Moderator as _,
            Arc::new(|bot, _, _| bot.reset_all()),
        );

        let backup_create = CommandBuilder::<Self>::new()
            .literal(["create"])
            .word()
            .finalize(
                true,
                AuthorityLevel::Moderator as _,
                Arc::new(|bot, _, args| match bot.backup_create(args[0].to_owned()) {
                    Ok(response) => response,
                    Err(err) => {
                        bot.log(LogType::Error, &format!("Failed to create backup: {err}"));
                        Some(format!("Failed to create backup"))
                    }
                }),
            );

        let backup = CommandBuilder::<Self>::new().literal(["create"]).finalize(
            true,
            AuthorityLevel::Moderator as _,
            Arc::new(|bot, _, _| match bot.backup_create("default") {
                Ok(response) => response,
                Err(err) => {
                    bot.log(LogType::Error, &format!("Failed to create backup: {err}"));
                    Some(format!("Failed to create backup"))
                }
            }),
        );

        let backup_load = CommandBuilder::<Self>::new()
            .literal(["load"])
            .word()
            .finalize(
                true,
                AuthorityLevel::Moderator as _,
                Arc::new(|bot, _, args| match bot.backup_load(args[0].to_owned()) {
                    Ok(response) => response,
                    Err(err) => {
                        bot.log(LogType::Error, &format!("Failed to load backup: {err}"));
                        Some(format!("Failed to load backup"))
                    }
                }),
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
            CommandBuilder::new()
                .literal(["!backup"])
                .split([backup_create, backup_load, backup]),
        ])
    }
}

fn clear_dir(path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    let path = path.as_ref();
    std::fs::remove_dir_all(path).unwrap_or(());
    std::fs::create_dir_all(path)?;
    Ok(())
}
