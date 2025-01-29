use crate::action::{ActionBuilder, ActionFeedback, ActionResult};
use std::pin::Pin;

pub struct DcaResponse;

pub fn preview_dca_action() -> Result<String, String> {}
pub fn execute_dca_action() -> Result<String, String> {}

pub async fn preview_dca(
    params: DcaParams,
    state: AgentState<S>,
) -> Result<ActionFeedback, String> {
    let preview = preview_dca_action()?;

    Ok(ActionFeedback {
        message: format!("DCA Preview: {}", preview),
        confirmation_data: serde_json::to_value(params)?,
    })
}

pub async fn execute_dca(params: DcaParams, state: AgentState<S>) -> Result<String, String> {
    execute_dca_action()
}

pub fn create_dca_action<S: Send + Sync + Clone + 'static>() -> FunctionAction<S> {
    ActionBuilder::<_, DcaParams, DcaParams, S>::new("dca", execute_dca, Some(preview_dca))
        .description("Dollar Cost Average into a token")
        .parameter("token", "Token to DCA into", "string", true)
        .parameter("amount", "Amount to invest", "number", true)
        .build()
}
