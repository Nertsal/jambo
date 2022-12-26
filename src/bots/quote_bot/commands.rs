use rand::seq::SliceRandom;

use super::*;

impl QuoteBot {
    fn quote_random(&self) -> Response {
        if let Some(random_quote_name) = self
            .config
            .quotes
            .keys()
            .collect::<Vec<&String>>()
            .choose(&mut rand::thread_rng())
        {
            Some(format!(
                "Quote {}: {}",
                random_quote_name, self.config.quotes[random_quote_name as &str]
            ))
        } else {
            Some("No quotes yet. Add new ones with !quote add <quote>".to_string())
        }
    }

    fn quote_new(&mut self, quote_name: String, quote: String) -> Response {
        if let std::collections::hash_map::Entry::Vacant(entry) =
            self.config.quotes.entry(quote_name.clone())
        {
            let response = Some(format!("Added new quote {}: {}", quote_name, quote));
            entry.insert(quote);
            self.config.save().unwrap();
            response
        } else {
            Some(format!(
                "A quote with the name {} already exists",
                quote_name
            ))
        }
    }

    fn quote_remove(&mut self, quote_name: &str) -> Response {
        match self.config.quotes.remove(quote_name) {
            Some(quote) => {
                self.config.save().unwrap();
                Some(format!("Deleted quote {:?}: {}", quote_name, quote))
            }
            None => Some(format!("I don't know any quote named {quote_name}. Try creating one with !quote new <quote_name> <quote>")),
        }
    }

    fn quote_edit(&mut self, quote_name: &str, new_quote: String) -> Response {
        match self.config.quotes.get_mut(quote_name) {
            Some(old_quote) => {
                let response = Some(format!(
                    "Edited quote {}: {}. New quote: {}",
                    quote_name, old_quote, new_quote
                ));
                *old_quote = new_quote;
                self.config.save().unwrap();
                response
            }
            None => {
                Some(format!("I don't know any quote named {quote_name}. Try creating one with !quote new <quote_name> <quote>"))
            }
        }
    }

    fn quote_get(&mut self, quote_name: &str) -> Response {
        match self.config.quotes.get(quote_name) {
            Some(quote) => {
                Some(quote.clone())
            }
            None => {
                Some(format!("I don't know any quote named {quote_name}. Try creating one with !quote new <quote_name> <quote>"))
            },
        }
    }

    fn quote_rename(&mut self, quote_name: &str, new_name: String) -> Response {
        if self.config.quotes.contains_key(&new_name) {
            Some(format!("A quote with name {} already exists", new_name))
        } else if let Some(quote) = self.config.quotes.remove(quote_name) {
            let response = Some(format!(
                "Changed quote's name from {} to {}",
                quote_name, new_name
            ));
            self.config.quotes.insert(new_name, quote);
            self.config.save().unwrap();
            response
        } else {
            Some(format!("No quote with name {} found", quote_name))
        }
    }

    pub fn commands() -> Commands<Self> {
        let random = CommandBuilder::<Self>::new().finalize(
            true,
            AuthorityLevel::Viewer as _,
            Arc::new(|bot, _, _| bot.quote_random()),
        );

        let new = CommandBuilder::<Self>::new()
            .literal(["new", "add"])
            .word()
            .line()
            .finalize(
                true,
                AuthorityLevel::Moderator as _,
                Arc::new(|bot, _, args| bot.quote_new(args[0].to_owned(), args[1].to_owned())),
            );

        let remove = CommandBuilder::<Self>::new()
            .literal(["delete", "remove"])
            .word()
            .finalize(
                true,
                AuthorityLevel::Moderator as _,
                Arc::new(|bot, _, args| bot.quote_remove(&args[0])),
            );

        let edit = CommandBuilder::<Self>::new()
            .literal(["edit"])
            .word()
            .line()
            .finalize(
                true,
                AuthorityLevel::Moderator as _,
                Arc::new(|bot, _, args| bot.quote_edit(&args[0], args[1].to_owned())),
            );

        let rename = CommandBuilder::<Self>::new()
            .literal(["rename"])
            .word()
            .word()
            .finalize(
                true,
                AuthorityLevel::Moderator as _,
                Arc::new(|bot, _, args| bot.quote_rename(&args[0], args[1].to_owned())),
            );

        let get = CommandBuilder::<Self>::new().word().line().finalize(
            true,
            AuthorityLevel::Viewer as _,
            Arc::new(|bot, _, args| bot.quote_get(&args[0])),
        );

        Commands {
            commands: vec![CommandBuilder::new()
                .literal(["!quote"])
                .split([random, new, remove, edit, rename, get])],
        }
    }
}
