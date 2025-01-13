use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionParameter {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct ActionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ActionParameter>,
}

/// An asynchronous action that an agent can perform.
/// It contains metadata about the action (description, parameters)
/// and the actual implementation (execute).
pub trait Action: Send + Sync {
    /// Returns the definition of this action, including its name,
    /// description, and parameters
    fn definition(&self) -> ActionDefinition;

    /// Executes the action with the given parameters
    fn execute(
        &self,
        params: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>>;
}

/// A builder to create actions from async functions
pub struct ActionBuilder<F> {
    name: String,
    description: String,
    parameters: Vec<ActionParameter>,
    handler: F,
}

impl<F, Fut> ActionBuilder<F>
where
    F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<String, String>> + Send + 'static,
{
    pub fn new(name: impl Into<String>, handler: F) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            parameters: Vec::new(),
            handler,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn parameter(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        param_type: impl Into<String>,
        required: bool,
    ) -> Self {
        self.parameters.push(ActionParameter {
            name: name.into(),
            description: description.into(),
            param_type: param_type.into(),
            required,
        });
        self
    }

    pub fn build(self) -> impl Action {
        FunctionAction {
            definition: ActionDefinition {
                name: self.name,
                description: self.description,
                parameters: self.parameters,
            },
            handler: Box::new(self.handler),
        }
    }
}

struct FunctionAction<F> {
    definition: ActionDefinition,
    handler: Box<F>,
}

impl<F, Fut> Action for FunctionAction<F>
where
    F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<String, String>> + Send + 'static,
{
    fn definition(&self) -> ActionDefinition {
        self.definition.clone()
    }

    fn execute(
        &self,
        params: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>> {
        Box::pin((self.handler)(params))
    }
}

// Example usage:
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_function_action() {
        async fn weather(params: serde_json::Value) -> Result<String, String> {
            let location = params["location"]
                .as_str()
                .ok_or("location parameter required")?;
            Ok(format!("Weather in {}: Sunny", location))
        }

        let action = ActionBuilder::new("get_weather", weather)
            .description("Get the weather for a location")
            .parameter("location", "The city to get weather for", "string", true)
            .build();

        // Test the definition
        let def = action.definition();
        assert_eq!(def.name, "get_weather");
        assert_eq!(def.parameters.len(), 1);

        // Test execution
        let params = serde_json::json!({
            "location": "London"
        });
        let result = action.execute(params).await.unwrap();
        assert_eq!(result, "Weather in London: Sunny");
    }
}
