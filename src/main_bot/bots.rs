use super::*;

pub(super) struct Bots {
    pub constructors: HashMap<BotName, BotConstructor>,
    pub active: HashMap<BotName, Box<dyn Bot>>,
}

impl Bots {
    pub fn new(cli: &Option<Cli>, active_bots: ActiveBots) -> Self {
        let constructors = constructors().into_iter().collect::<HashMap<_, _>>();
        let mut active = HashMap::new();
        for bot_name in active_bots {
            match constructors.get(&bot_name) {
                Some(constructor) => {
                    let bot = constructor(cli);
                    log(cli, LogType::Info, &format!("Spawned {bot_name}"));
                    active.insert(bot_name, bot);
                }
                None => {
                    log(
                        cli,
                        LogType::Warn,
                        &format!("Failed to find a constructor for {bot_name}"),
                    );
                }
            }
        }
        Self {
            constructors,
            active,
        }
    }
}
