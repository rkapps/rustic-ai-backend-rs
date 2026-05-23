use anyhow::Result;
use async_trait::async_trait;
use rustic_core::Tool;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};


#[derive(Debug, Serialize, Clone)]
pub struct OrchestratorStageDecision {}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDecision {
    pub agents: Vec<String>, // agent_ids from available_agents pool
    pub execution: ExecutionMode,
    pub stop: bool,
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    Sequential,
    Parallel,
}


#[async_trait]
impl Tool for OrchestratorStageDecision {
    fn name(&self) -> String {
        "decide_next_stage".to_string()
    }

    fn description(&self) -> String {
        "Decide which agents to run next".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        let parameters = json!({
                "type": "object",
                "properties": {
                "agents": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of agent ids to invoke"
                },
                "execution": {
                    "type": "string",
                    "enum": ["Parallel", "Sequential"],
                    "description": "How to execute the agents"
                },
                "stop": {
                    "type": "boolean",
                    "description": "Set to true when enough data has been gathered"
                },
                "reasoning": {
                    "type": "string",
                    "description": "Why these agents were chosen"
                }
            },
            "required": ["agents", "execution", "stop"]
        });

        parameters
    }

    async fn execute(&self, value: serde_json::Value) -> Result<Value> {
        // let stage_decision: StageDecision = serde_json::from_value(value.clone())
        //     .map_err(|e| anyhow::anyhow!("Error deserializing arguments {:#?}: {}", value, e))?;
        // info!("STage decison: {:?}", stage_decision);
        Ok(value)
    }
}
