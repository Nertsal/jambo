use super::*;

impl CustomBot {
    fn command_new(&mut self, command_name: String, command_response: String) -> Response {
        if self.config.commands.contains_key(&command_name) {
            Some(format!(
                "A command with that name already exists. Try !edit {command_name} <new response>"
            ))
        } else {
            let response = Some(format!(
                "Added new command: {command_name}: {command_response}"
            ));
            self.command_edit(command_name, command_response);
            self.config.save().unwrap();
            response
        }
    }

    fn command_delete(&mut self, command_name: &str) -> Response {
        match self.config.commands.remove(command_name) {
            Some(command_response) => {
                self.remove_command(command_name);
                self.config.save().unwrap();
                Some(format!(
                    "Removed the command: {command_name}: {command_response}"
                ))
            }
            None => Some(format!("A command with that name does not exist")),
        }
    }

    fn command_edit(&mut self, command_name: String, command_response: String) -> Response {
        let response = Some(format!(
            "Updated command to {command_name}: {command_response}"
        ));
        self.config
            .commands
            .insert(command_name.clone(), command_response);
        self.remove_command(&command_name);
        self.push_command(command_name);
        self.config.save().unwrap();
        response
    }

    fn remove_command(&mut self, command_name: &str) {
        self.commands.commands.retain(|command| match command {
            CommandNode::Literal { literals, .. } => {
                !literals.iter().any(|literal| *literal == *command_name)
            }
            _ => true,
        })
    }

    pub fn push_command(&mut self, command_name: String) {
        self.commands.commands.push(CommandNode::Literal {
            literals: vec![command_name.clone()],
            child_nodes: vec![CommandNode::final_node(
                true,
                AuthorityLevel::Viewer as usize,
                Arc::new(move |bot, _, _| Some(bot.config.commands[&command_name].clone())),
            )],
        });
    }

    pub fn commands() -> Commands<Self> {
        let new = CommandBuilder::<Self, _>::new()
            .literal(["new"])
            .word()
            .line()
            .finalize(
                true,
                AuthorityLevel::Moderator as _,
                Arc::new(|bot, _, args| bot.command_new(args[0].to_owned(), args[1].to_owned())),
            );

        let delete = CommandBuilder::<Self, _>::new()
            .literal(["delete", "remove"])
            .word()
            .finalize(
                true,
                AuthorityLevel::Moderator as _,
                Arc::new(|bot, _, args| bot.command_delete(&args[0])),
            );

        let edit = CommandBuilder::<Self, _>::new()
            .literal(["edit"])
            .word()
            .line()
            .finalize(
                true,
                AuthorityLevel::Moderator as _,
                Arc::new(|bot, _, args| bot.command_edit(args[0].to_owned(), args[1].to_owned())),
            );

        Commands {
            commands: vec![CommandBuilder::new()
                .literal(["!command"])
                .split(vec![new, delete, edit])],
        }
    }
}
