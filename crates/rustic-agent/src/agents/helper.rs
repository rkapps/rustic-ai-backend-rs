use rustic_core::{HttpError, HttpResult};
use tracing::trace;

use crate::{CompletionResponse, Message, agents::StageDecision};

pub fn build_agent_messages(response: CompletionResponse) -> Vec<Message> {
    let mut messages = Vec::new();
    if let Some(text) = response.text() {
        if !text.trim().is_empty() {
            // guard against empty

            let clean = text
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim()
                .to_string();

            messages.push(Message::Assistant {
                content: clean,
                response_id: Some(response.response_id),
            });
        }
    }
    messages
}

pub fn build_stage_decision(response: CompletionResponse) -> HttpResult<StageDecision> {
    let content = response.text();
    if let Some(val) = content {
        trace!("val: {}", val);
        let clean = val
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        match serde_json::from_str::<StageDecision>(clean) {
            Ok(decision) => return Ok(decision),
            Err(e) => Err(HttpError::Other(format!(
                "Failed to parse StageDecision: {}",
                e
            ))),
        }
    } else {
        return Err(HttpError::Other(
            "Failed to parse completion response".to_string(),
        ));
    }
}
