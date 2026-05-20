//! High-level services that wire configuration, registries, and the agent builder together.
//!
//! The typical call flow is:
//!
//! ```text
//! ProviderRegistry + AgentRegistry + ToolRegistry + MCPRegistry
//!         └─► AgentService::from_registry(...)
//!                     └─► AgentService::builder()  →  AgentBuilder
//!                                 └─► .with_provider(...).with_preset(...).build()
//!                                                              └─► Agent
//! ```
//!
//! - [`config`] — deserialised JSON config types for providers, agents, and MCP servers
//! - [`registry`] — in-memory registries populated from config at startup
//! - [`agent`] — [`AgentService`](agent::AgentService): the primary entry point for building agents
//! - [`builder`] — [`AgentBuilder`](builder::AgentBuilder): fluent builder returned by `AgentService::builder()`

pub mod agent;
pub mod builder;
pub mod config;
pub mod registry;
