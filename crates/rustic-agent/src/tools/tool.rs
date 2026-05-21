use std::{collections::HashMap, sync::Arc};

use rustic_core::Tool;

/// In-process tool registry used by [`Agent`](crate::agents::Agent) to dispatch tool calls.
///
/// Tools are keyed by their [`Tool::name`](crate::client::tools::Tool::name) and stored behind
/// `Arc<dyn Tool>` so they can be cheaply cloned across async tasks.
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    pub registry: HashMap<String, Arc<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    /// Create an empty registry.
    pub fn new() -> ToolRegistry {
        Self {
            registry: HashMap::new(),
        }
    }

    /// Register a tool from a pre-wrapped `Arc<dyn Tool>`.
    pub fn register_tool_arc(&mut self, tool: Arc<dyn Tool>) {
        self.registry.insert(tool.name(), tool);
    }

    /// Register a statically-typed tool; the concrete type is erased into `Arc<dyn Tool>`.
    pub fn register_tool<T: Tool + 'static>(&mut self, tool: T) {
        self.registry.insert(tool.name(), Arc::new(tool));
    }

    /// Register a tool from an `Arc<dyn Tool>` (alias of [`register_tool_arc`](Self::register_tool_arc)
    /// kept for ergonomics when registering from a `Vec`).
    pub fn register_tool_boxed(&mut self, tool: Arc<dyn Tool>) {
        self.registry.insert(tool.name().to_string(), tool);
    }

    /// Look up a tool by name. Returns `None` if the tool has not been registered.
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.registry.get(name).cloned()
    }

    /// Return all registered tools (order is unspecified).
    pub fn get_tools(&self) -> Vec<Arc<dyn Tool>> {
        self.registry.values().cloned().collect()
    }
}
