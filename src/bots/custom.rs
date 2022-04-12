use super::*;

pub struct CustomBot {
    cli: Cli,
    commands: Commands<Self>,
}

impl Bot<Self> for CustomBot {
    fn inner(&mut self) -> &mut Self {
        self
    }

    fn commands(&self) -> &Commands<Self> {
        &self.commands
    }
}

impl CustomBot {
    pub fn new(cli: &Cli) -> Self {
        Self {
            cli: cli.clone(),
            commands: Commands::new(vec![]),
        }
    }
}
