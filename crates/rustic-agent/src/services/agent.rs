use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, trace};

use tokio::sync::RwLock;

use crate::{
    agents::Agent,
    client::{llm::LlmClient, preset::Preset, provider::Provider},
    services::{
        builder::AgentBuilder,
        registry::{agent::AgentRegistry, provider::ProviderRegistry},
    },
    tools::{mcp::MCPRegistry, tool::ToolRegistry},
};

/// Central service for constructing [`Agent`] instances at request time.
///
/// `AgentService` holds shared references to all registries and a cache of
/// live [`LlmClient`] instances keyed by `"{provider}:{model}"`. Clients are
/// created on first use and reused across subsequent builds, avoiding repeated
/// initialisation overhead.
///
/// Use [`builder`](Self::builder) for full control, or the higher-level
/// [`build_chat_agent`](Self::build_chat_agent) and [`build_agent_for_id`](Self::build_agent_for_id)
/// for the common cases.
#[derive(Clone)]
pub struct AgentService {
    /// Live LLM client cache keyed by `"{LLM}:{model}"`.
    pub clients: Arc<RwLock<HashMap<String, Arc<dyn LlmClient>>>>,
    pub agent_registry: Arc<AgentRegistry>,
    pub provider_registry: Arc<ProviderRegistry>,
    pub tool_registry: Arc<RwLock<ToolRegistry>>,
    pub mcp_registry: Arc<RwLock<MCPRegistry>>,
}

impl AgentService {
    /// Construct the service from pre-built registries.
    ///
    /// The client cache starts empty and is populated lazily as agents are built.
    pub fn from_registry(
        provider_registry: Arc<ProviderRegistry>,
        agent_registry: Arc<AgentRegistry>,
        tool_registry: Arc<RwLock<ToolRegistry>>,
        mcp_registry: Arc<RwLock<MCPRegistry>>,
    ) -> Self {
        Self {
            agent_registry,
            provider_registry,
            clients: Arc::new(RwLock::new(HashMap::new())),
            tool_registry,
            mcp_registry,
        }
    }

    /// Return a new [`AgentBuilder`] borrowing from this service.
    pub fn builder(&self) -> AgentBuilder<'_> {
        AgentBuilder::new(self)
    }

    /// Build a general-purpose chat agent for the given provider and model.
    ///
    /// Uses `Preset::Balanced` for hosted providers and `Preset::Local` for
    /// `Provider::Local`. The `system_prompt` defaults to an empty string if `None`.
    pub async fn build_chat_agent(
        &self,
        llm: &str,
        model: &str,
        system_prompt: Option<String>,
    ) -> Result<Agent> {
        let provider = self.resolve_provider(llm, Some(model))?;

        let preset = match &provider {
            Provider::Local { .. } => Preset::Local,
            _ => Preset::Balanced,
        };
        let system_prompt = system_prompt.unwrap_or_default();
        debug!("System Prompt: {}", system_prompt);
        debug!("Preset: {:?}", preset);

        let agent = self
            .builder()
            .with_system_prompt(system_prompt)
            .with_preset(preset)
            .with_provider(provider)
            .await?
            .build()
            .await?;

        Ok(agent)
    }

    /// Build an agent configured by a pre-registered [`AgentConfig`].
    ///
    /// Looks up `agent_id` in the [`AgentRegistry`], filters the global tool registry
    /// down to only the tools listed in the agent's config, then builds with the
    /// agent's system prompt and preset. Returns an error if `agent_id` is not found.
    pub async fn build_agent_for_id(
        &self,
        agent_id: &str,
        llm: &str,
        model: &str,
    ) -> Result<Agent> {
        let agent_config = self
            .agent_registry
            .find(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", agent_id))?;

        debug!("Agent Config: {}", agent_config.id);
        let provider = self.resolve_provider(llm, Some(model))?;
        let preset = match &provider {
            Provider::Local { .. } => Preset::Local,
            _ => Preset::Balanced,
        };

        let tool_registry = {
            let global = self.tool_registry.read().await;
            let mut filtered = ToolRegistry::new();
            for tool_id in &agent_config.tools {
                trace!("Tool: {:?}", tool_id);
                if let Some(tool) = global.get_tool(tool_id) {
                    filtered.register_tool_arc(tool);
                }
            }
            filtered
        };
        trace!("System Prompt: {}", agent_config.system_prompt);
        debug!("Tools: {}", tool_registry.get_tools().len());
        debug!("Preset: {:?}", preset);

        let agent = self
            .builder()
            .with_system_prompt(agent_config.system_prompt.clone())
            .with_tools(tool_registry.get_tools())
            .with_preset(preset)
            .with_provider(provider)
            .await?
            .build()
            .await?;

        Ok(agent)
    }

    /// Resolve a [`Provider`] enum variant from a provider ID and optional model override.
    ///
    /// Falls back to the provider's `default_model` when `model` is `None`.
    /// Unknown provider IDs are treated as local/Ollama servers and require `base_url` to be set.
    ///
    /// Returns an error if the provider is not registered, the required API key is missing,
    /// or a local provider has no `base_url` configured.
    pub fn resolve_provider(&self, id: &str, model: Option<&str>) -> anyhow::Result<Provider> {
        debug!("Resolve Provider: llm: {:?} model: {:?}", id, model);
        let provider = self
            .provider_registry
            .find(id)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", id))?;

        let model = model.unwrap_or(&provider.default_model);

        match id {
            "openai" => Ok(Provider::openai(
                self.provider_registry
                    .get_api_key("openai")
                    .ok_or_else(|| anyhow::anyhow!("OpenAI API key not configured"))?,
                model,
            )),
            "gemini" => Ok(Provider::gemini(
                self.provider_registry
                    .get_api_key("gemini")
                    .ok_or_else(|| anyhow::anyhow!("Gemini API key not configured"))?,
                model,
            )),
            "anthropic" => Ok(Provider::anthropic(
                provider
                    .api_key
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("Anthropic API key not configured"))?,
                model,
            )),
            _ => {
                let base_url = provider.base_url.as_deref().ok_or_else(|| {
                    anyhow::anyhow!("Provider '{}' has no base_url configured", id)
                })?;

                Ok(Provider::local(model, base_url))
            }
        }
    }
}
