use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A JSON-RPC 2.0 request envelope used to communicate with MCP tool servers.
///
/// `id` and `params` are omitted from serialization when `None` so that
/// notification messages (which must not carry an `id`) remain spec-compliant.
#[derive(Debug, Serialize)]
pub struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    /// Omitted for notifications (no response expected).
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

impl JsonRpcRequest {
    /// Create a request with explicit control over every field.
    ///
    /// Pass `id: None` to send a JSON-RPC *notification* (no reply expected).
    pub fn new(jsonrpc: String, method: String, id: Option<Value>, params: Option<Value>) -> Self {
        Self {
            jsonrpc,
            method,
            id,
            params,
        }
    }

    /// Create a request using JSON-RPC 2.0 defaults (`jsonrpc: "2.0"`, `id: 1`).
    pub fn default(method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: String::from("2.0"),
            method,
            id: serde_json::from_str("1").unwrap(),
            params,
        }
    }
}

/// A JSON-RPC 2.0 response envelope returned by an MCP tool server.
///
/// `T` is the expected shape of the `result` field for a given method.
/// Error responses are not modelled here; callers should check HTTP status
/// codes before deserializing into this type.
#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse<T> {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: Value,
    /// The structured result payload for a successful call.
    pub result: T,
}
