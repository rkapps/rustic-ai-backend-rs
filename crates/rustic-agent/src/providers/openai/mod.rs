//! OpenAI provider: request mapping to the Responses API, streaming SSE parsing, and response normalisation.

pub mod completion;
pub mod request;
pub mod response;

/// Provider identifier returned by [`models`] callers for display or routing.
pub const LLM: &str = "OpenAI";

pub const MODEL_GPT_5_NANO: &str = "gpt-5-nano";
pub const MODEL_GPT_5_4_NANO: &str = "gpt-5.4-nano";
pub const MODEL_GPT_5_4_MINI: &str = "gpt-5.4-mini";
pub const MODEL_GPT_5_4: &str = "gpt-5.4";

pub const MODEL_TEXT_EMBEDDING_3_SMALL: &str = "text-embedding-3-small";
const OPENAI_BASE_URL: &str = "https://api.openai.com";

/// Return the list of supported GPT model identifiers.
pub fn models() -> Vec<String> {
    vec![
        MODEL_GPT_5_4_MINI.to_string(),
        MODEL_GPT_5_4_NANO.to_string(),
        MODEL_GPT_5_4.to_string(),
    ]
}
