use anyhow::Result;
use serde_json::Value;
use std::fmt::Debug;

use async_trait::async_trait;

/// A callable tool that the agent can invoke during a completion loop.
///
/// Implementors describe themselves via `name`, `description`, and `parameters`
/// (a JSON Schema object). The agent core converts these into [`ToolDefinition`]
/// values and passes them to the LLM; when the model requests a call the agent
/// dispatches it to `execute`.
#[async_trait]
pub trait Tool: Send + Sync + Debug {
    /// Unique identifier used to route calls from the model to this tool.
    fn name(&self) -> String;
    /// Human-readable description sent to the model to explain what the tool does.
    fn description(&self) -> String;
    /// JSON Schema object describing the tool's accepted parameters.
    fn parameters(&self) -> serde_json::Value;
    /// Execute the tool with the given `value` arguments and return the result.
    async fn execute(&self, value: serde_json::Value) -> Result<Value>;
}
