use anyhow::Result;
use tracing::{debug, info};

use crate::{Agent, CompletionResponse, CompletionResponseContent, Message};

pub struct Orchestrator {
    agent: Agent,
}

impl Orchestrator {
    pub fn new(agent: Agent) -> Orchestrator {
        Self { agent }
    }

    pub async fn execute(&self, messages: &[Message]) -> Result<CompletionResponse> {
        let response = self.agent.complete(&messages).await?;
        debug!("Respoinse: {:?}", response);
        for content in response.clone().contents {
            if let CompletionResponseContent::Text(val) = content {
                info!("{:#?}", val);
            }
        }
        Ok(response)
    }


}
