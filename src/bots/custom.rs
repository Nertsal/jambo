use super::*;

pub struct CustomBot {
    cli: Cli,
    commands: Commands<Self>,
}

impl Bot<Self> for CustomBot {
    const NAME: &'static str = "CustomBot";

    fn inner(&mut self) -> &mut Self {
        self
    }

    fn commands(&self) -> &Commands<Self> {
        &self.commands
    }
}

impl CustomBot {
    pub fn subbot(cli: &Cli) -> SubBot {
        SubBot::Custom(Self {
            cli: cli.clone(),
            commands: Commands::new(vec![]),
        })
    }
}
