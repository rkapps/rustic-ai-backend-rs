//! Text embedding clients.
//!
//! [`client`] defines the [`client::EmbeddingClient`] trait and the shared
//! [`client::Embedding`] / [`client::BatchResult`] types.  The remaining
//! modules are concrete implementations:
//!
//! | Module   | Backend              | Requires                  |
//! |----------|----------------------|---------------------------|
//! | [`candle`] | Local BERT via Candle | Model files on disk      |
//! | [`gemini`] | Google Gemini API   | `GEMINI_API_KEY` env var  |
//! | [`openai`] | OpenAI Embeddings API | `OPENAI_API_KEY` env var |

pub mod candle;
pub mod client;
pub mod gemini;
pub mod openai;
