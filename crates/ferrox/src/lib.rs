pub mod action;
pub mod agent;
use teloxide::Bot;

pub struct Ferrox {
    bot: Bot,
}

impl Ferrox {
    pub fn new() -> Self {
        Self {
            bot: Bot::from_env(),
        }
    }
}
