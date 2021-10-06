use super::*;

impl CommandBot<Self, Sender> for GameJamBot {
    fn get_commands(&self) -> &Commands<Self, Sender> {
        &self.commands
    }

    fn get_cli(&self) -> &CLI {
        &self.cli
    }
}

impl GameJamBot {
    pub fn commands() -> Commands<Self, Sender> {
        Commands { commands: vec![] }
    }
}
