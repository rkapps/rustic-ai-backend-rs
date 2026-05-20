use serde::Deserialize;
use serde_json::Value;

/// Top-level response from `POST /v1beta/interactions`.
#[derive(Debug, Deserialize)]
pub struct GeminiInteractionsResponse {
    /// Provider-assigned interaction ID (used as `previous_interaction_id` in follow-up turns).
    pub id: String,
    pub model: String,
    /// Ordered output items produced by the model.
    pub outputs: Vec<GeminiInteractionsResponseOutput>,
    pub status: String,
    pub usage: GeminiInteractionsResponseTokenUsage,
}

/// A single output item in a Gemini response, discriminated by `type`.
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum GeminiInteractionsResponseOutput {
    /// Plain text from the model.
    #[serde(rename = "text")]
    Text { text: String },

    /// Extended thinking output; `signature` is an opaque blob that must be
    /// echoed back to Gemini in subsequent turns.
    #[serde(rename = "thought")]
    Thought { signature: String },

    /// A function/tool call requested by the model.
    #[serde(rename = "function_call")]
    FunctionCall {
        id: String,
        arguments: Value,
        name: String,
    },
}

/// Detailed token accounting returned by the Gemini Interactions API.
#[derive(Deserialize, Debug)]
pub struct GeminiInteractionsResponseTokenUsage {
    pub total_tokens: i32,
    pub total_input_tokens: i32,
    /// Tokens served from the prompt cache.
    pub total_cached_tokens: i32,
    /// Tokens attributed to tool definitions and results.
    pub total_tool_use_tokens: i32,
    /// Tokens consumed by internal chain-of-thought reasoning.
    pub total_thought_tokens: i32,
    /// Visible output tokens (excluding thought tokens).
    pub total_output_tokens: i32,
}

/// A single SSE event in a Gemini streaming response.
#[derive(Debug, Deserialize)]
pub struct GeminiInteractionsChunkResponse {
    /// Event type: `"content.delta"` for incremental output, `"interaction.complete"` for the final event.
    pub event_type: String,
    /// Present on `content.delta` events.
    pub delta: Option<GeminiInteractionsChunkResponseDelta>,
    /// Present on `interaction.complete` events; carries final usage stats.
    pub interaction: Option<GeminiInteractionsChunkResponseInteraction>,
}

/// An incremental content delta within a streaming response.
#[derive(Debug, Deserialize)]
pub struct GeminiInteractionsChunkResponseDelta {
    /// Delta type: `"text"`, `"thought"`, or `"function_call"`.
    pub r#type: String,
    /// Incremental text for `text` deltas.
    pub text: Option<String>,
    /// Thought signature blob for `thought` deltas.
    pub signature: Option<String>,
    /// Tool call ID for `function_call` deltas.
    pub id: Option<String>,
    /// Tool name for `function_call` deltas.
    pub name: Option<String>,
    /// Tool arguments for `function_call` deltas.
    pub arguments: Option<Value>,
}

/// Final interaction metadata delivered on the `interaction.complete` event.
#[derive(Debug, Deserialize)]
pub struct GeminiInteractionsChunkResponseInteraction {
    pub model: String,
    pub id: String,
    pub usage: Option<GeminiInteractionsResponseTokenUsage>,
}

/// Legacy generate-content response shape (kept for compatibility).
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiResponseCandidate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiResponseCandidate {
    pub content: GeminiResponseContent,
}

#[derive(Debug, Deserialize)]
pub struct GeminiResponseContent {
    pub parts: Vec<GeminiResponseContentPart>,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct GeminiResponseContentPart {
    pub text: String,
}
