use serde::{de::DeserializeOwned, Deserialize, Serialize};
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
pub trait Action: Send + Sync {
    fn definition(&self) -> ActionDefinition;
    fn execute(
        &self,
        params: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>>;
}

/// A builder to create actions from async functions with typed parameters
pub struct ActionBuilder<F, P> {
    name: String,
    description: String,
    parameters: Vec<ActionParameter>,
    handler: F,
    _phantom: std::marker::PhantomData<P>,
}

impl<F, Fut, P> ActionBuilder<F, P>
where
    F: Fn(P) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = Result<String, String>> + Send + Sync + 'static,
    P: DeserializeOwned + Send + 'static,
{
    pub fn new(name: impl Into<String>, handler: F) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            parameters: Vec::new(),
            handler,
            _phantom: std::marker::PhantomData,
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
        let handler = self.handler;
        FunctionAction {
            definition: ActionDefinition {
                name: self.name,
                description: self.description,
                parameters: self.parameters,
            },
            handler: Box::new(move |params: serde_json::Value| {
                let handler = handler.clone();
                async move {
                    let params = serde_json::from_value(params)
                        .map_err(|e| format!("Invalid parameters: {}", e))?;
                    handler(params).await
                }
            }),
        }
    }
}

struct FunctionAction<F> {
    definition: ActionDefinition,
    handler: Box<F>,
}

impl<F, Fut> Action for FunctionAction<F>
where
    F: Fn(serde_json::Value) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<String, String>> + Send + Sync + 'static,
{
    fn definition(&self) -> ActionDefinition {
        self.definition.clone()
    }

    fn execute(
        &self,
        params: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>> {
        Box::pin((self.handler)(params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Define a strongly typed parameter struct
    #[derive(Debug, Deserialize)]
    struct WeatherParams {
        location: String,
        #[serde(default)]
        units: Option<String>,
    }

    #[tokio::test]
    async fn test_typed_function_action() {
        async fn weather(params: WeatherParams) -> Result<String, String> {
            let units = params.units.unwrap_or_else(|| "celsius".to_string());
            Ok(format!("Weather in {} ({}): Sunny", params.location, units))
        }

        let action = ActionBuilder::<_, WeatherParams>::new("get_weather", weather)
            .description("Get the weather for a location")
            .parameter("location", "The city to get weather for", "string", true)
            .parameter(
                "units",
                "Temperature units (celsius/fahrenheit)",
                "string",
                false,
            )
            .build();

        // Test the definition
        let def = action.definition();
        assert_eq!(def.name, "get_weather");
        assert_eq!(def.parameters.len(), 2);

        // Test execution with all parameters
        let params = serde_json::json!({
            "location": "London",
            "units": "fahrenheit"
        });
        let result = action.execute(params).await.unwrap();
        assert_eq!(result, "Weather in London (fahrenheit): Sunny");

        // Test execution with only required parameters
        let params = serde_json::json!({
            "location": "Paris"
        });
        let result = action.execute(params).await.unwrap();
        assert_eq!(result, "Weather in Paris (celsius): Sunny");

        // Test execution with invalid parameters
        let params = serde_json::json!({
            "wrong_field": "London"
        });
        let result = action.execute(params).await;
        assert!(result.is_err());
    }
}
