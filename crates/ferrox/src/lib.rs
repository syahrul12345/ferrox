pub mod agent;

use std::{collections::HashMap, sync::Arc};

use agent::Agent;
use ferrox_actions::ConfirmHandler;
use teloxide::{
    dispatching::UpdateHandler,
    prelude::*,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
    RequestError,
};
use tokio::sync::Mutex;

pub struct Ferrox<A, S>
where
    A: Agent<S> + Send + Sync + Clone + 'static,
    S: Send + Sync + Clone + 'static,
{
    bot: Bot,
    agent: A,
    callback_data: Arc<Mutex<HashMap<String, (serde_json::Value, ConfirmHandler<S>)>>>,
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
            callback_data: Arc::new(Mutex::new(HashMap::new())),
            _state: std::marker::PhantomData,
        }
    }

    /// Starts the Telegram bot and handles incoming messages
    pub async fn start(&self) {
        let bot = self.bot.clone();
        let agent = Arc::new(self.agent.clone());
        let callback_data = self.callback_data.clone();
        let message_handler = move |bot: Bot, msg: Message| {
            let agent = agent.clone();
            let callback_data = callback_data.clone();
            async move {
                if let Some(text) = msg.text() {
                    let history_id = msg.chat.id.to_string();
                    let sent_message = bot.send_message(msg.chat.id, "Thinking...").await?;
                    println!("event=PROCESSING_PROMPT");
                    match agent.process_prompt(text, &history_id).await {
                        Ok((response, confirm_handler)) => {
                            println!("event=RECEIVE_RESPONSE_FROM_AGENT: {:?}", response);
                            // Check if this is a preview response that needs confirmation
                            if let Some((value, confirm_handler)) = confirm_handler {
                                // Create confirm/cancel buttons
                                let uuid = uuid::Uuid::new_v4().to_string();
                                let keyboard = InlineKeyboardMarkup::new(vec![vec![
                                    InlineKeyboardButton::callback("Confirm", uuid.clone()),
                                ]]);

                                // Store the preview data for later confirmation
                                // You might want to store this in a HashMap in the agent
                                bot.edit_message_text(
                                    sent_message.chat.id,
                                    sent_message.id,
                                    response,
                                )
                                .reply_markup(keyboard)
                                .await?;
                                callback_data
                                    .lock()
                                    .await
                                    .insert(uuid, (value, confirm_handler));
                            } else {
                                bot.edit_message_text(
                                    sent_message.chat.id,
                                    sent_message.id,
                                    response,
                                )
                                .await?;
                            }
                        }
                        Err(e) => {
                            println!("Error processing prompt");
                            println!("Error: {:?}", e);
                            bot.send_message(msg.chat.id, "Error processing prompt")
                                .await?;
                        }
                    }
                }
                Ok::<(), RequestError>(())
            }
        };
        println!("event=MESSAGE_HANDLER_CREATED");
        let agent = Arc::new(self.agent.clone());
        let callback_data = self.callback_data.clone();
        let callback_handler = move |bot: Bot, q: CallbackQuery| {
            let callback_data = callback_data.clone();
            let agent = agent.clone();

            async move {
                if let Some(data) = q.data {
                    // Get the stored data and handler
                    if let Some((value, handler)) = callback_data.lock().await.remove(&data) {
                        // Execute the confirmation handler
                        let result = handler(value, agent.state()).await;
                        match result {
                            Ok(response) => {
                                // Update the message with the confirmation result
                                if let Some(message) = q.message {
                                    bot.edit_message_text(message.chat.id, message.id, response)
                                        .await?;
                                }
                            }
                            Err(e) => {
                                println!("Error handling confirmation: {:?}", e);
                                if let Some(message) = q.message {
                                    bot.edit_message_text(
                                        message.chat.id,
                                        message.id,
                                        "Error processing confirmation",
                                    )
                                    .await?;
                                }
                            }
                        }
                    }
                }

                // Answer the callback query to remove the loading state
                bot.answer_callback_query(q.id).await?;
                Ok::<(), RequestError>(())
            }
        };
        println!("event=CALLBACK_HANDLER_CREATED");

        //Start the bot
        let message_handler = Update::filter_message().branch(dptree::endpoint(message_handler));
        let callback_handler = Update::filter_callback_query().endpoint(callback_handler);
        let handler = dptree::entry()
            .branch(message_handler)
            .branch(callback_handler);
        println!("event=STARTING_TELEGRAM_BOT");
        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    }
}
