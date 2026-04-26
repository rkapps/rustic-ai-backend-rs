use std::sync::Arc;
use anyhow::Result;
use agentic_core::agent::service::LlmProvider;
use axum::{Json, extract::State, response::IntoResponse};
use reqwest::StatusCode;
use rustic_ai_services::rustic::RusticService;

use crate::state::AppState;

pub async fn get_llm_providers_handler(
    State(analyse_service): State<Arc<RusticService>>,
) -> Result<Json<Vec<LlmProvider>>, (StatusCode, String)> {
    let providers = analyse_service.get_llm_providers();
    Ok(Json(providers))
}

pub async fn get_templates_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    Json(state.templates.clone())
}