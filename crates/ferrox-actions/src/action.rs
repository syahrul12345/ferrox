use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{future::Future, pin::Pin, sync::Arc};

use crate::AgentState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmptyParams {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionParameter {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ActionParameter>,
}

pub struct FunctionAction<S: Send + Sync + Clone + 'static> {
    definition: ActionDefinition,
    handler: Box<
        dyn Fn(
                serde_json::Value,
                AgentState<S>,
            ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>>
            + Send
            + Sync,
    >,
    confirm_handler: Option<
        Box<
            dyn Fn(
                    serde_json::Value,
                    AgentState<S>,
                )
                    -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>>
                + Send
                + Sync,
        >,
    >,
}

impl<S: Send + Sync + Clone + 'static> FunctionAction<S> {
    pub fn definition(&self) -> ActionDefinition {
        self.definition.clone()
    }

    pub fn execute(
        &self,
        params: serde_json::Value,
        state: AgentState<S>,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>> {
        (self.handler)(params, state)
    }

    pub fn confirm(
        &self,
        params: serde_json::Value,
        state: AgentState<S>,
    ) -> Option<Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>>> {
        self.confirm_handler
            .as_ref()
            .map(|handler| handler(params, state))
    }
}

pub type EmptyConfirmHandler<T, S> =
    fn(T, AgentState<S>) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>>;
/// A builder to create actions from async functions with typed parameters
/// F: The handler function type
/// P: Input parameters for the handler
/// S: State type
/// Q: Output type from handler and input type for confirm handler
/// CF: Confirm handler function type (defaults to EmptyConfirmHandler)
pub struct ActionBuilder<F, P, S, Q = String, CF = EmptyConfirmHandler<Q, S>> {
    name: String,
    description: String,
    parameters: Vec<ActionParameter>,
    handler: F,
    confirm_handler: Option<CF>,
    _phantom_handler_input: std::marker::PhantomData<P>,
    _phantom_confirm_handler_input: std::marker::PhantomData<Q>,
    _phantom_state: std::marker::PhantomData<S>,
}

impl<F, CF, P, Q, S, Fut, CFut> ActionBuilder<F, P, S, Q, CF>
where
    // Handler F takes P and returns Q
    F: Fn(P, AgentState<S>) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = Result<Q, String>> + Send + Sync + 'static,
    P: DeserializeOwned + Send + 'static,
    // Confirm handler CF takes Q and returns String
    CF: Fn(Q, AgentState<S>) -> CFut + Send + Sync + Clone + 'static,
    CFut: Future<Output = Result<String, String>> + Send + Sync + 'static,
    Q: Serialize + DeserializeOwned + Send + 'static,
    S: Send + Sync + Clone + 'static,
{
    pub fn new(name: impl Into<String>, handler: F, confirm_handler: Option<CF>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            parameters: Vec::new(),
            handler,
            confirm_handler,
            _phantom_handler_input: std::marker::PhantomData,
            _phantom_confirm_handler_input: std::marker::PhantomData,
            _phantom_state: std::marker::PhantomData,
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

    pub fn build(self) -> FunctionAction<S> {
        let handler = self.handler;
        let confirm_handler = self.confirm_handler.clone();
        FunctionAction {
            definition: ActionDefinition {
                name: self.name,
                description: self.description,
                parameters: self.parameters,
            },
            handler: Box::new(move |params: serde_json::Value, state: AgentState<S>| {
                let handler = handler.clone();
                let confirm_handler = confirm_handler.clone();
                Box::pin(async move {
                    let params = serde_json::from_value(params)
                        .map_err(|e| format!("Invalid parameters: {}", e))?;
                    let result = handler(params, state).await?;

                    // If there's a confirm handler, mark this as a preview
                    if confirm_handler.is_some() {
                        let result_str = serde_json::to_string(&result)
                            .map_err(|e| format!("Failed to serialize result: {}", e))?;
                        Ok(format!("PREVIEW:\n{}", result_str))
                    } else {
                        // No confirm handler, just serialize the result
                        serde_json::to_string(&result)
                            .map_err(|e| format!("Failed to serialize result: {}", e))
                    }
                })
            }),
            confirm_handler: self.confirm_handler.clone().map(|handler| {
                Box::new(move |params: serde_json::Value, state: AgentState<S>| {
                    let handler = handler.clone();
                    let fut: Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>> =
                        Box::pin(async move {
                            let params = serde_json::from_value::<Q>(params)
                                .map_err(|e| format!("Invalid parameters: {}", e))?;
                            handler(params, state).await
                        });
                    fut
                })
                    as Box<
                        dyn Fn(
                                serde_json::Value,
                                AgentState<S>,
                            ) -> Pin<
                                Box<dyn Future<Output = Result<String, String>> + Send + Sync>,
                            > + Send
                            + Sync,
                    >
            }),
        }
    }
}

/// Represents a group of related actions
pub trait ActionGroup<S: Send + Sync + Clone + 'static> {
    fn actions(&self) -> &[Arc<FunctionAction<S>>];
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::Mutex;

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
        async fn weather(params: WeatherParams, _state: AgentState<()>) -> Result<String, String> {
            let units = params.units.unwrap_or_else(|| "celsius".to_string());
            Ok(format!("Weather in {} ({}): Sunny", params.location, units))
        }

        let action = ActionBuilder::<_, WeatherParams, ()>::new("get_weather", weather, None)
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
        let state = Arc::new(Mutex::new(()));
        let params = serde_json::json!({
            "location": "London",
            "units": "fahrenheit"
        });
        let result = action.execute(params, state.clone()).await.unwrap();
        assert_eq!(result, "Weather in London (fahrenheit): Sunny");

        // Test execution with only required parameters
        let params = serde_json::json!({
            "location": "Paris"
        });
        let result = action.execute(params, state.clone()).await.unwrap();
        assert_eq!(result, "Weather in Paris (celsius): Sunny");

        // Test execution with invalid parameters
        let params = serde_json::json!({
            "wrong_field": "London"
        });
        let result = action.execute(params, state.clone()).await;
        assert!(result.is_err());
    }
}
