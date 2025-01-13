use std::{future::Future, pin::Pin};

use super::Agent;

pub struct TextAgent<T>
where
    T: Agent,
{
    pub inner_agent: Option<T>,
    pub system_prompt: String,
}

impl<T: Agent> Agent for TextAgent<T> {
    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    //Process the prompt
    fn process_prompt(
        &self,
        prompt: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>>>> {
        todo!()
    }
}
