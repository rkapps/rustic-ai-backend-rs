use serde::Serialize;
use serde_json::Value;

/// JSON-RPC params body for a `tools/list` request (empty — no parameters needed).
#[derive(Debug, Serialize)]
pub(super) struct MCPToolListRequest {}

/// JSON-RPC params body for a `tools/get` request.
#[derive(Debug, Serialize)]
pub(super) struct MCPToolGetParamsRequest {
    pub(super) tool_name: String,
}

/// JSON-RPC params body for a `tools/call` request.
#[derive(Debug, Serialize)]
pub(super) struct MCPToolCallParamsRequest {
    /// Name of the tool to invoke.
    pub(super) name: String,
    /// Arguments forwarded directly to the tool.
    pub(super) arguments: Value,
}
