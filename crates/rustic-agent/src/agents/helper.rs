//! Utility functions for building and transforming pipeline messages.
//!
//! These helpers sit between the raw [`CompletionResponse`] values returned by individual agents
//! and the message history maintained by [`PipeLineRunner`]. They handle JSON fence stripping,
//! sub-agent message construction, orchestrator decision parsing, and context merging.

use rustic_core::{HttpError, HttpResult};
use tracing::trace;

use crate::{
    CompletionResponse, Message,
    agents::{StageDecision, SubAgentResponse},
};

/// Concatenate the trailing run of `Assistant` messages into a single string.
///
/// Walks `messages` from the end and collects consecutive `Assistant` entries
/// (stopping at the first non-`Assistant` message). The collected content is joined
/// with double newlines and returned as the merged pipeline stage output.
pub fn build_merged_sub_agent_message(messages: &mut Vec<Message>) -> String {
    let merged = messages
        .iter()
        .rev()
        .take_while(|m| matches!(m, Message::Assistant { .. }))
        .collect::<Vec<_>>()
        .iter()
        .rev()
        .filter_map(|m| match m {
            Message::Assistant { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    merged
}

/// Append a labelled `Assistant` message to `messages` from a sub-agent's [`CompletionResponse`].
///
/// The message body is a JSON object `{"agent": "<id>", "content": <value>}` so the orchestrator
/// can attribute outputs to specific agents. If the response contains no text, nothing is pushed.
pub fn build_sub_agent_messages(messages: &mut Vec<Message>, response: &CompletionResponse) {
    if let Some(sub_response) = build_sub_agent_response(response) {
        // parse content as JSON value first
        let content_value: serde_json::Value = serde_json::from_str(&sub_response.content)
            .unwrap_or(serde_json::Value::String(sub_response.content.clone()));

        let content = serde_json::json!({
            "agent": sub_response.agent_id,
            "content": content_value  // embedded as value not string
        }).to_string();

        messages.push(Message::Assistant {
            content,
            response_id: None,
        });
    }
}

/// Extract a [`SubAgentResponse`] from a [`CompletionResponse`], stripping JSON fences.
///
/// Returns `None` if the response contains no text content.
pub fn build_sub_agent_response(response: &CompletionResponse) -> Option<SubAgentResponse> {
    // if is_orchestrator_decision_response(response) {
    //     return None;
    // }
    response.text().map(|t| SubAgentResponse {
        agent_id: response.id.clone(),
        content: build_clean_json(t),
    })
}

/// Unwrap the inner `"content"` value from a labelled agent JSON message.
///
/// Pipeline messages are stored as `{"agent": "...", "content": ...}`. This function
/// returns the raw content string for use as input to the next stage. Falls back to
/// returning `content` unchanged if it is not valid JSON or lacks the `"content"` key.
pub fn unwrap_agent_content(content: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(inner) = v.get("content") {
            // handle both string and object
            match inner {
                serde_json::Value::String(s) => s.clone(),
                _ => inner.to_string(), // serialize object back to string
            }
        } else {
            content.to_string()
        }
    } else {
        content.to_string()
    }
}


/// Strip markdown code fences (` ```json ` or ` ``` `) from LLM output.
///
/// Many models wrap JSON responses in fences even when instructed not to; this normalises
/// the text before deserialisation.
pub fn build_clean_json(text: &str) -> String {
    text.trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string()
}

/// Return `true` if `m` is the orchestrator's "decide next stage" prompt.
///
/// Used to filter decide-prompts out of context windows passed to sub-agents so they
/// don't see the orchestration scaffolding.
pub fn is_decide_prompt(m: &Message) -> bool {
    matches!(m, Message::User { content, .. } 
        if content.starts_with("Based on the above, decide"))
}

/// Deserialise a [`StageDecision`] from the orchestrator's [`CompletionResponse`].
///
/// Strips JSON fences before parsing. Returns [`HttpError::Other`] if the response
/// contains no text or the text cannot be parsed as a valid [`StageDecision`].
pub fn build_stage_decision(response: CompletionResponse) -> HttpResult<StageDecision> {
    let content = response.text();
    if let Some(val) = content {
        trace!("val: {}", val);
        let clean = &build_clean_json(val);

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

/// Return `true` if `m` is an `Assistant` message whose content is an orchestrator decision JSON.
///
/// A message qualifies when it is valid JSON containing both `"agents"` and `"stop"` keys,
/// which is the minimum shape of a [`StageDecision`].
pub fn is_orchestrator_decision(m: &Message) -> bool {
    match m {
        Message::Assistant { content, .. } => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(content) {
                // must have agents array and stop field to be a decision
                v.get("agents").is_some() && v.get("stop").is_some()
            } else {
                false
            }
        }
        _ => false,
    }
}
