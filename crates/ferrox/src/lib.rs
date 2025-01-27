pub mod agent;

use std::sync::Arc;

use agent::Agent;
use teloxide::{prelude::*, types::Message};

pub struct Ferrox<A, S>
where
    A: Agent<S> + Send + Sync + Clone + 'static,
    S: Send + Sync + Clone + 'static,
{
    bot: Bot,
    agent: A,
    _state: std::marker::PhantomData<S>,
}

impl<A, S> Ferrox<A, S>
where
    A: Agent<S> + Send + Sync + Clone + 'static,
    S: Send + Sync + Clone + 'static,
{
    pub fn new(agent: A) -> Self {
        Self {
            bot: Bot::from_env(),
            agent,
            _state: std::marker::PhantomData,
        }
    }

    /// Starts the Telegram bot and handles incoming messages
    pub async fn start(&self) {
        let bot = self.bot.clone();
        let agent = Arc::new(self.agent.clone());

        teloxide::repl(bot, move |bot: Bot, msg: Message| {
            let agent = agent.clone();
            async move {
                if let Some(text) = msg.text() {
                    let history_id = msg.chat.id.to_string();
                    match agent.process_prompt(text, &history_id).await {
                        Ok(response) => {
                            bot.send_message(msg.chat.id, response).await?;
                        }
                        Err(e) => {
                            println!("Error processing prompt");
                            println!("Error: {:?}", e);
                            bot.send_message(msg.chat.id, "Error processing prompt")
                                .await?;
                        }
                    }
                }
                Ok(())
            }
        })
        .await;
    }
}
