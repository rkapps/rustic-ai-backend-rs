use anyhow::Result;
use axum::extract::FromRef;
use bin_shared::templates::config::TemplateConfig;
use rustic_ai_services::{chat::ChatsService, rustic::RusticService};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::middleware::firebase_auth::fetch_firebase_keys;

pub struct FirebaseKeyCache {
    pub keys: HashMap<String, String>,
    pub fetched_at: std::time::Instant,
}

impl FirebaseKeyCache {
    pub fn is_stale(&self) -> bool {
        // Firebase keys are valid for ~1 hour, refresh every 55 min
        self.fetched_at.elapsed().as_secs() > 55 * 60
    }
}

#[derive(Clone)]
pub struct AppState {
    pub chats_service: Arc<ChatsService>,
    pub rustic_service: Arc<RusticService>,
    pub firebase_keys: Arc<RwLock<FirebaseKeyCache>>,
    pub templates: Option<TemplateConfig>
}

impl AppState {
    pub async fn new(
        chats_service: Arc<ChatsService>,
        rustic_service: Arc<RusticService>,
        templates: Option<TemplateConfig>
    ) -> Result<Self> {
        let keys = fetch_firebase_keys().await?;
        Ok(Self {
            chats_service,
            rustic_service,
            firebase_keys: Arc::new(RwLock::new(FirebaseKeyCache {
                keys,
                fetched_at: std::time::Instant::now(),
            })),
            templates
        })
    }
}

impl FromRef<AppState> for Arc<ChatsService> {
    fn from_ref(state: &AppState) -> Self {
        state.chats_service.clone()
    }
}

impl FromRef<AppState> for Arc<RusticService> {
    fn from_ref(state: &AppState) -> Self {
        state.rustic_service.clone()
    }
}


impl FromRef<AppState> for Arc<Option<TemplateConfig>> {
    fn from_ref(state: &AppState) -> Self {
        state.templates.clone().into()
    }
}