pub mod action;
pub mod agent;

use std::sync::Arc;

use agent::Agent;
use teloxide::{prelude::*, types::Message};

pub struct Ferrox<P> {
    bot: Bot,
    agent: P,
}

impl<P: Agent + Clone + Send + Sync + 'static> Ferrox<P> {
    pub fn new(agent: P) -> Self {
        Self {
            bot: Bot::from_env(),
            agent: agent.clone(),
        }
    }

    /// Starts the Telegram bot and handles incoming messages
    pub async fn start(self) {
        let bot = self.bot;
        let agent = Arc::new(self.agent);

        teloxide::repl(bot, move |bot: Bot, msg: Message| {
            let agent = agent.clone();
            async move {
                if let Some(text) = msg.text() {
                    let history_id = msg.chat.id.to_string();
                    if let Ok(response) = agent.process_prompt(text, &history_id).await {
                        bot.send_message(msg.chat.id, response).await?;
                    }
                }
                Ok(())
            }
        })
        .await;
    }
}
