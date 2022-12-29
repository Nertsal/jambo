use super::*;

pub struct MutexBot(Mutex<MainBot>);

impl MutexBot {
    pub fn new(bot: MainBot) -> Self {
        Self(Mutex::new(bot))
    }
}

impl std::ops::Deref for MutexBot {
    type Target = Mutex<MainBot>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl linefeed::Completer<linefeed::DefaultTerminal> for MutexBot {
    fn complete(
        &self,
        word: &str,
        prompter: &Prompter,
        start: usize,
        end: usize,
    ) -> Option<Vec<linefeed::Completion>> {
        let mut main = futures::executor::block_on(self.0.lock());
        let main_completetion = main.commands.complete(word, prompter, start, end);
        let bots = &mut main.bots;

        let mut completions = vec![main_completetion];
        completions.extend(
            bots.active
                .values()
                .map(|bot| bot.complete(word, prompter, start, end)),
        );

        Some(completions.into_iter().flatten().flatten().collect())
    }
}
