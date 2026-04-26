use anyhow::Result;
use serde::Deserialize;

use crate::chats::chat::Chat;

#[derive(Deserialize, Debug)]
pub struct ChatConfig {
    pub llm: Option<String>,
    pub model: Option<String>,
    pub title: Option<String>,
    pub system: Option<String>,
    pub prompt: Option<String>,
    pub stream: bool,
}

impl ChatConfig {
    pub fn validate(self) -> Result<Chat> {
        let llm = self
            .llm
            .ok_or_else(|| anyhow::anyhow!("Llm cannot be blank."))?;
        let model = self
            .model
            .ok_or_else(|| anyhow::anyhow!("Model cannot be blank"))?;
        let title = self
            .title
            .ok_or_else(|| anyhow::anyhow!("Title cannot be blank"))?;
        let prompt = self
            .prompt
            .ok_or_else(|| anyhow::anyhow!("Prompt cannot be blank"))?;
        let stream = self.stream;

        Ok(Chat::new(
            "".to_string(),
            llm,
            model,
            title,
            self.system,
            prompt,
            stream,
        ))
    }
}
