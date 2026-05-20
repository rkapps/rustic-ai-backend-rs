//! In-memory registries populated from config at startup and consulted at request time.
//!
//! - [`AgentRegistry`](agent::AgentRegistry) — stores [`AgentConfig`](super::config::agent::AgentConfig) entries; supports lookup, catalog filtering, and pipeline sub-agent queries
//! - [`ProviderRegistry`](provider::ProviderRegistry) — stores resolved [`ResolvedProvider`](super::config::provider::ResolvedProvider) entries with decrypted API keys and base URLs

pub mod agent;
pub mod provider;
