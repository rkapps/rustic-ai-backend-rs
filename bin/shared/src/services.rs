use std::{env, sync::Arc};

use agentic_core::agent::config::{AgentServiceConfig, LocalEndpoint};
use anyhow::Result;
use rustic_ai_storage::{
    mongo::{manager::MongoStorageManager, service::MongoStorageService},
    service::StorageService,
};

use crate::config::app_config::AppConfig;

pub async fn load_agent_config(app_config: &AppConfig) -> AgentServiceConfig {
    let mut config = AgentServiceConfig::default();

    for provider in &app_config.llm.providers {
        if !provider.enabled {
            continue;
        }

        if provider.id == "local" {
            // local needs base_url from env
            if let Some(base_url_env) = &provider.base_url_env {
                if let Ok(base_url) = std::env::var(base_url_env) {
                    config.local_endpoints.push(LocalEndpoint {
                        id: provider.id.clone(),
                        label: provider.label.clone(),
                        base_url,
                        default_model: provider.default_model.clone(),
                        models: provider.models.clone(),
                    });
                }
            }
        } else {
            // all others - read api key from env var
            if let Some(api_key_env) = &provider.api_key_env {
                if let Ok(key) = std::env::var(api_key_env) {
                    config.api_keys.insert(provider.id.clone(), key);
                }
            }
        }
    }

    config
}


// Returns the storage service
pub async fn get_storage_service() -> Result<Arc<dyn StorageService>> {
    let storage_manager = get_mongo_manager().await?;
    Ok(Arc::new(MongoStorageService::new(storage_manager)))
}

// Returns the storage manager
async fn get_mongo_manager() -> Result<MongoStorageManager> {
    let mongo_uri = env::var("MONGO_ATLAS_CONN_STR")
        .expect("MONGO_ATLAS_CONN_STR envrionment variable not set");
    println!("MongoAtlas Uri: {}", mongo_uri);

    MongoStorageManager::new(&mongo_uri, "rustic-ai").await
}
