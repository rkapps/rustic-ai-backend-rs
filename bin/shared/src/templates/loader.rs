use anyhow::Result;

use crate::{gcs::download_gcs_string, templates::config::TemplateConfig};

pub async fn load_templates() -> Result<TemplateConfig> {
    let path = std::env::var("TEMPLATES_PATH")
        .unwrap_or_else(|_| "bin/shared/src/templates/templates.json".to_string());

    let content = if path.starts_with("gs://") {
        download_gcs_string(&path).await?
    } else {
        std::fs::read_to_string(&path)?
    };

    Ok(serde_json::from_str(&content)?)
}