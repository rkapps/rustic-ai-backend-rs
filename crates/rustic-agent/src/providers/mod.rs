//! LLM provider implementations.
//!
//! Each sub-module wraps a specific provider's HTTP API and implements
//! [`LlmClient`](crate::client::llm::LlmClient) so the rest of the crate
//! can remain provider-agnostic.
//!
//! | Module       | Provider           |
//! |--------------|--------------------|
//! | [`anthropic`] | Anthropic (Claude) |
//! | [`openai`]    | OpenAI             |
//! | [`gemini`]    | Google Gemini      |
//! | [`local`]     | Local / Ollama     |

pub mod anthropic;
pub mod gemini;
pub mod local;
pub mod openai;
