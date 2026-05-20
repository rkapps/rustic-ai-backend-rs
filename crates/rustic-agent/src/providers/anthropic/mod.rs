//! Anthropic (Claude) provider: request mapping, streaming SSE parsing, and response normalisation.

pub mod completion;
pub mod request;
pub mod response;

/// Provider identifier returned by [`models`] callers for display or routing.
pub const LLM: &str = "Anthropic";

pub const MODEL_CLAUDE_SONNET_4_5: &str = "claude-sonnet-4-5";
pub const MODEL_CLAUDE_SONNET_4_6: &str = "claude-sonnet-4-6";
pub const MODEL_CLAUDE_HAIKU_4_5: &str = "claude-haiku-4-5";
pub const MODEL_CLAUDE_OPUS_4_6: &str = "claude-opus-4-6";

const ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Return the list of supported Claude model identifiers.
pub fn models() -> Vec<String> {
    vec![
        MODEL_CLAUDE_SONNET_4_5.to_string(),
        MODEL_CLAUDE_SONNET_4_6.to_string(),
        MODEL_CLAUDE_HAIKU_4_5.to_string(),
        MODEL_CLAUDE_OPUS_4_6.to_string(),
    ]
}
