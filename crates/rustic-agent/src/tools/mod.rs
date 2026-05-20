//! Tool management: registries for local tools and remote MCP server tools.
//!
//! - [`tool`] — [`ToolRegistry`](tool::ToolRegistry) for in-process [`Tool`](crate::client::tools::Tool) implementations
//! - [`mcp`] — [`MCPRegistry`](mcp::MCPRegistry) and [`MCPClient`](mcp::MCPClient) for remote MCP tool servers
//! - [`request`] / [`response`] — internal MCP wire types (not part of the public API)

pub mod mcp;
pub mod request;
pub mod response;
pub mod tool;
