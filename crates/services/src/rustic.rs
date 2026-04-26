use anyhow::Result;
use std::sync::Arc;
use tracing::debug;

use agentic_core::agent::{
    builder::Preset,
    completion::Agent,
    provider::Provider,
    service::{AgentService, LlmProvider},
};

#[derive(Clone)]
pub struct RusticService {
    agent_service: Arc<AgentService>,
}

impl RusticService {
    pub fn new(agent_service: Arc<AgentService>) -> Self {
        Self { agent_service }
    }

    /// Returns configured LLM providers — UI uses this for dropdown
    pub fn get_llm_providers(&self) -> Vec<LlmProvider> {
        self.agent_service.get_llm_providers()
    }

    pub fn get_chat_agent(&self, llm: &str, model: &str) -> Result<Agent> {
        debug!("LLm: {} Model: {}", llm, model);
        let provider = self.agent_service.resolve_provider(llm, Some(model))?;
        debug!("Provider: {:?}", provider);
        // For a non local agent, use thorough
        let preset = match &provider {
            Provider::Local { .. } => Preset::Local,
            _ => Preset::Balanced,
        };

        let agent = self
            .agent_service
            .builder()
            .with_preset(preset)
            .with_provider(provider)?
            .build()?;

        Ok(agent)
    }
}
