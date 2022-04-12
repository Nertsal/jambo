use super::*;

pub struct CustomBot {
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
    pub fn new() -> Self {
        Self {
            commands: Commands::new(vec![]),
        }
    }
}
