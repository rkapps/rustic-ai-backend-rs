use serde::Deserialize;
use serde_json::Value;

/// Top-level response from `POST /v1/messages`.
#[derive(Debug, Deserialize)]
pub struct AnthropicCompletionResponse {
    pub model: String,
    pub role: String,
    /// Ordered list of content blocks produced by the model.
    pub content: Vec<AnthropicCompletionResponseContent>,
    pub usage: AnthropicCompletionResponseTokenUsage,
}

/// A single content block in an Anthropic response, discriminated by `type`.
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum AnthropicCompletionResponseContent {
    /// Plain text output.
    #[serde(rename = "text")]
    Text { text: String },

    /// Extended thinking / chain-of-thought block.
    #[serde(rename = "thinking")]
    Thought { thinking: String },

    /// A tool-use request from the model.
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        input: Value,
        name: String,
    },
}

/// Token accounting returned by Anthropic for both blocking and streaming calls.
#[derive(Deserialize, Clone, Debug)]
pub struct AnthropicCompletionResponseTokenUsage {
    /// Fresh (non-cached) input tokens.
    pub input_tokens: i32,
    /// Tokens written to the prompt cache on this call.
    pub cache_creation_input_tokens: Option<i32>,
    /// Tokens read from the prompt cache on this call.
    pub cache_read_input_tokens: Option<i32>,
    pub output_tokens: i32,
}

/// A single SSE event in an Anthropic streaming response.
#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicChunkResponse {
    /// Event type (e.g. `"content_block_start"`, `"content_block_delta"`, `"message_delta"`).
    pub r#type: String,
    /// Content-block index; present on block-level events.
    pub index: Option<i32>,
    pub delta: Option<AnthropicChunkResponseDelta>,
    /// Token usage; present on `message_delta` (final) events.
    pub usage: Option<AnthropicCompletionResponseTokenUsage>,
    /// Present on `content_block_start` events.
    pub content_block: Option<AnthropicChunkResponseContentBlock>,
}

/// The opening descriptor for a content block (`content_block_start`).
#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicChunkResponseContentBlock {
    /// Block type: `"text"`, `"thinking"`, or `"tool_use"`.
    pub r#type: String,
    pub text: Option<String>,
    /// Tool call ID; present when `type == "tool_use"`.
    pub id: Option<String>,
    /// Tool name; present when `type == "tool_use"`.
    pub name: Option<String>,
    pub input: Option<Value>,
    pub caller: Option<Value>,
}

/// Incremental update within a content block (`content_block_delta`).
#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicChunkResponseDelta {
    /// Delta type: `"text_delta"`, `"input_json_delta"`, or `"thinking_delta"`.
    pub r#type: Option<String>,
    /// Incremental text for `text_delta`.
    pub text: Option<String>,
    /// Incremental thinking text for `thinking_delta`.
    pub thinking: Option<String>,
    /// Incremental JSON fragment for `input_json_delta` (tool argument streaming).
    pub partial_json: Option<String>,
    pub usage: Option<AnthropicCompletionResponseTokenUsage>,
}
