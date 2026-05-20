//! Deserialised configuration types loaded from JSON files at startup.
//!
//! These types mirror the shape of the config files in `RUSTIC_AI_CONFIG_PATH`:
//!
//! | Module | File |
//! |---|---|
//! | [`agent`] | `agents.json` |
//! | [`provider`] | `providers.json` |
//! | [`mcp`] | `mcp_servers_config.json` |

pub mod agent;
pub mod mcp;
pub mod provider;
