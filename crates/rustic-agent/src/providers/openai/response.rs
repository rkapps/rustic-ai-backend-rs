use serde::Deserialize;
use serde_json::Value;

/// Top-level response from `POST /v1/responses`.
#[derive(Deserialize, Debug)]
pub struct OpenAICompletionResponse {
    /// Provider-assigned response ID (used as `previous_response_id` in follow-up turns).
    pub id: String,
    pub model: String,
    /// Ordered output items produced by the model.
    pub output: Vec<OpenAICompletionResponseOutput>,
    pub usage: OpenAICompletionResponseTokenUsage,
}

/// A single output item in an OpenAI response, discriminated by `type`.
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum OpenAICompletionResponseOutput {
    /// A text message from the model.
    #[serde(rename = "message")]
    Message {
        id: String,
        status: String,
        content: Vec<OpenAICompletionResponseContent>,
    },

    /// A function/tool call requested by the model.
    #[serde(rename = "function_call")]
    FunctionCall {
        status: String,
        /// JSON-encoded arguments string.
        arguments: String,
        call_id: String,
        name: String,
    },

    /// An internal reasoning item (not surfaced to the caller).
    #[serde(rename = "reasoning")]
    Reasoning { id: String, summary: Vec<String> },
}

/// A content block within a `Message` output item.
#[derive(Deserialize, Debug)]
pub struct OpenAICompletionResponseContent {
    /// Content type; `"output_text"` carries the visible model reply.
    pub r#type: String,
    pub text: String,
}

/// Token accounting returned by OpenAI for both blocking and streaming calls.
#[derive(Deserialize, Debug)]
pub struct OpenAICompletionResponseTokenUsage {
    pub total_tokens: i32,
    pub input_tokens: i32,
    pub input_tokens_details: OpenAICompletionResponseInputToken,
    pub output_tokens_details: OpenAICompletionResponseOutputToken,
    pub output_tokens: i32,
}

/// Breakdown of input token costs.
#[derive(Deserialize, Debug)]
pub struct OpenAICompletionResponseInputToken {
    /// Tokens served from the prompt cache.
    pub cached_tokens: i32,
}

/// Breakdown of output token costs.
#[derive(Deserialize, Debug)]
pub struct OpenAICompletionResponseOutputToken {
    /// Tokens consumed by internal chain-of-thought reasoning.
    pub reasoning_tokens: i32,
}

/// Outer SSE envelope (unused in the current streaming path, kept for completeness).
#[derive(Debug, Deserialize)]
pub struct OpentAIChunkResponse {
    pub event: String,
    pub data: Option<OpenAIChunkResponseData>,
}

/// The data payload of a single SSE event in an OpenAI streaming response.
#[derive(Debug, Deserialize)]
pub struct OpenAIChunkResponseData {
    /// SSE event type (e.g. `"response.output_text.delta"`, `"response.completed"`).
    pub r#type: String,
    /// Present on `response.completed` events; carries final usage stats.
    pub response: Option<OpenAIChunkResponseDataResponse>,
    /// Incremental text fragment for `response.output_text.delta`.
    pub delta: Option<String>,
    /// Present on `response.output_item.added` for new function-call items.
    pub item: Option<OpenAIChunkResponseDataItem>,
    /// Item ID used to correlate `delta` events to a specific output item.
    pub item_id: Option<String>,
}

/// A new output item announced during streaming (e.g. a function call being initiated).
#[derive(Debug, Deserialize)]
pub struct OpenAIChunkResponseDataItem {
    pub id: String,
    /// Item type: `"function_call"`, `"message"`, etc.
    pub r#type: String,
    pub status: Option<String>,
    pub arguments: Option<Value>,
    pub call_id: Option<String>,
    pub name: Option<String>,
}

/// Final response metadata delivered on the `response.completed` event.
#[derive(Debug, Deserialize)]
pub struct OpenAIChunkResponseDataResponse {
    pub id: String,
    pub model: String,
    /// Final token usage for the full request.
    pub usage: Option<OpenAICompletionResponseTokenUsage>,
}
