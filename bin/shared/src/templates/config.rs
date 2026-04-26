use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTemplate {
    pub id: String,
    pub category: String,
    pub label: String,
    pub description: String,
    pub system_prompt: Option<String>,
    pub suggested_prompts: Vec<String>,
    pub tools: Vec<String>,
    pub template_type: TemplateType,
    pub recommended_llm: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateType {
    Chat,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub templates: Vec<ChatTemplate>,
}

