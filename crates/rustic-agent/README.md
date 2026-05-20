# rustic-agent

Multi-provider LLM client library for Rust. Provides a unified agent abstraction over Anthropic (Claude), OpenAI (GPT), Google Gemini, and local Ollama-compatible servers with support for streaming, tool use, and MCP servers.

## Features

- **Multi-provider** — Anthropic, OpenAI, Gemini, Local/Ollama behind a single `LlmClient` trait
- **Agentic tool loop** — Automatic multi-iteration tool dispatch (blocking and streaming), up to 10 rounds with concurrency control and per-call timeouts
- **MCP support** — Connect to Model Context Protocol servers with a pluggable adapter interface
- **Streaming** — SSE streaming for all providers via `complete_with_stream` and `complete_with_tools_streaming`
- **Extended thinking** — Reasoning/thinking token support for Anthropic and Gemini
- **Prompt caching** — Cache-control header integration for Anthropic
- **Token accounting** — Detailed breakdown: input, cached read/write, tool, reasoning, and output tokens

## Module Layout

```
rustic-agent
├── agents/       — Agent struct and completion loop orchestration
├── client/       — Provider-agnostic traits and types
│   ├── llm.rs    — LlmClient trait
│   ├── message.rs — Message enum (User, Assistant, Thought, ToolCall, ToolOutput)
│   ├── request.rs — CompletionRequest, ReasoningEffort
│   ├── response.rs — CompletionResponse, CompletionChunkResponse, token usage
│   ├── tools.rs  — Tool trait, ToolDefinition, ToolCallRequest
│   ├── mcp.rs    — MCPServerAdapter trait
│   └── rpc.rs    — JSON-RPC 2.0 envelope types
├── providers/
│   ├── anthropic/ — Claude via Messages API
│   ├── openai/    — GPT via Responses API
│   ├── gemini/    — Gemini via Interactions API
│   └── local/     — Anthropic-compatible local servers (Ollama)
└── tools/
    ├── tool.rs    — ToolRegistry
    └── mcp.rs     — MCPRegistry, MCPClient, StandardAdapter
```

## Usage

### Creating an Agent

```rust
use std::sync::Arc;
use rustic_agent::{
    agents::Agent,
    client::{message::Message, request::ReasoningEffort},
    providers::anthropic::{completion::AnthropicClient, MODEL_CLAUDE_SONNET_4_6},
    tools::{mcp::MCPRegistry, tool::ToolRegistry},
};

let agent = Agent {
    client: Arc::new(AnthropicClient::new(api_key)?),
    llm: "anthropic".to_string(),
    model: MODEL_CLAUDE_SONNET_4_6.to_string(),
    system_prompt: Some("You are a helpful assistant.".to_string()),
    temperature: 0.7,
    max_tokens: 4096,
    enable_cache: true,
    reasoning_effort: ReasoningEffort::Medium,
    tool_registry: Arc::new(ToolRegistry::new()),
    mcp_registry: Arc::new(MCPRegistry::new()),
};
```

### Completion Methods

| Method | Tools | Streaming |
|---|---|---|
| `complete(&messages)` | No | No |
| `complete_with_stream(&messages)` | No | Yes |
| `complete_with_tools(&messages)` | Yes | No |
| `complete_with_tools_streaming(&messages)` | Yes | Yes |

### Messages

```rust
// Standard turns
Message::User      { content, response_id }
Message::Assistant { content, response_id }

// Managed automatically by the tool loop
Message::Thought   { content }
Message::ToolCall  { call_id, name, arguments }
Message::ToolOutput { call_id, name, output }
```

`response_id` threads conversation state for providers that require it (OpenAI `previous_response_id`, Gemini `previous_interaction_id`).

### Custom Tools

```rust
use async_trait::async_trait;
use rustic_agent::client::tools::Tool;

#[derive(Debug)]
struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> String { "get_weather".to_string() }
    fn description(&self) -> String { "Get current weather for a city".to_string() }
    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": { "city": { "type": "string" } },
            "required": ["city"]
        })
    }
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({ "temperature": "22°C", "city": args["city"] }))
    }
}

let mut registry = ToolRegistry::new();
registry.register_tool(WeatherTool);
```

### MCP Servers

```rust
use rustic_agent::tools::mcp::{MCPRegistry, MCPServerConfig};

let mut mcp = MCPRegistry::new();

// Initialises the session and fetches the tool list
let definitions = mcp.register_server(MCPServerConfig {
    name: "docs".to_string(),
    url: "http://localhost:8081/mcp".to_string(),
    api_key: "".to_string(),
}).await?;

// Register individual tools with full parameter schemas
// Stored as "docs___search" to avoid name collisions
mcp.register_tool("docs", "search").await?;

// Or bulk-register all tools from the server
mcp.add_definitions("docs", definitions);
```

### Streaming

```rust
use tokio_stream::StreamExt;

let mut stream = agent.complete_with_tools_streaming(&messages).await?;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if chunk.is_final {
        println!("Usage: {:?}", chunk.usage);
        break;
    }
    print!("{}", chunk.content);
}
```

### Providers

```rust
// Anthropic — Messages API
AnthropicClient::new(api_key)?
AnthropicClient::new_with_base_url(api_key, version, base_url)?  // custom endpoint

// OpenAI — Responses API
OpenAIClient::new(api_key)?

// Google Gemini — Interactions API
GeminiClient::new(api_key)?

// Local / Ollama — Anthropic-compatible
LocalClient::anthropic_compat("http://localhost:11434".to_string())?
```

## Supported Models

### Anthropic

| Constant | Model |
|---|---|
| `MODEL_CLAUDE_SONNET_4_6` | `claude-sonnet-4-6` |
| `MODEL_CLAUDE_SONNET_4_5` | `claude-sonnet-4-5` |
| `MODEL_CLAUDE_HAIKU_4_5` | `claude-haiku-4-5` |
| `MODEL_CLAUDE_OPUS_4_6` | `claude-opus-4-6` |

### OpenAI

| Constant | Model |
|---|---|
| `MODEL_GPT_5_4` | `gpt-5.4` |
| `MODEL_GPT_5_4_MINI` | `gpt-5.4-mini` |
| `MODEL_GPT_5_4_NANO` | `gpt-5.4-nano` |
| `MODEL_TEXT_EMBEDDING_3_SMALL` | `text-embedding-3-small` |

### Gemini

| Constant | Model |
|---|---|
| `MODEL_GEMINI_3_FLASH_PREVIEW` | `gemini-3-flash-preview` |
| `MODEL_GEMINI_3_1_FLASH_LITE_PREVIEW` | `gemini-3.1-flash-lite-preview` |
| `MODEL_GEMINI_3_1_PRO_PREVIEW` | `gemini-3.1-pro-preview` |
| `MODEL_GEMINI_EMBEDDING_001` | `gemini-embedding-001` |
