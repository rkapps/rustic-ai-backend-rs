//! High-level agent orchestration: drives the LLM completion loop and dispatches tool calls.

pub mod agent;
pub mod helper;
pub mod pipeline_runner;

pub use agent::Agent;
pub use pipeline_runner::PipeLineRunner;
use serde::{Deserialize, Serialize};

/// Orchestrator's parsed decision for a single pipeline stage.
///
/// The orchestrator LLM returns this as JSON. The pipeline runner deserialises it,
/// runs the chosen agents, and then loops back to ask for the next decision — unless
/// `stop` is `true`, in which case the final agent's output becomes the pipeline response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDecision {
    /// IDs of agents to run in this stage, drawn from the pipeline's `available_agents` pool.
    pub agents: Vec<String>,
    pub execution: ExecutionMode,
    /// When `true`, this is the final stage; the runner returns after executing these agents.
    pub stop: bool,
    /// Optional explanation from the orchestrator (useful for debugging multi-hop reasoning).
    pub reasoning: Option<String>,
    /// If set, overrides the user input forwarded to every agent in this stage.
    pub goal: Option<String>,
}

/// Controls whether agents in a stage run one-after-another or concurrently.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    /// Agents execute in order; each receives the previous agent's output as context.
    Sequential,
    /// Agents execute concurrently (bounded by a semaphore); results are merged afterwards.
    Parallel,
}

/// Normalised output from a sub-agent, ready to be merged into the pipeline conversation.
#[derive(Debug, Clone)]
pub struct SubAgentResponse {
    pub agent_id: String,
    /// Plain text content — JSON fences and orchestrator decision payloads are stripped.
    pub content: String,
}
