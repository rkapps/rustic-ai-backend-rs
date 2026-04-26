use serde::{Deserialize, Serialize};
use storage_core::core::RepoModel;
use uuid::Uuid;

use crate::chats::CHAT_COLLECTION_NAME;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chat {
    pub uid: String,
    pub id: String,
    pub title: String,
    pub llm: String,
    pub model: String,
    pub system: Option<String>,
    pub prompt: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub response_id: String,
}

impl RepoModel<String> for Chat {
    fn id(&self) -> String {
        self.clone().id
    }
    fn collection(&self) -> &'static str {
        CHAT_COLLECTION_NAME
    }
}

impl Chat {
    pub fn new(
        uid: String,
        llm: String,
        model: String,
        title: String,
        system: Option<String>,
        prompt: String,
        stream: bool,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            uid,
            id: id,
            title: title,
            llm: llm,
            model: model,
            system: system,
            prompt: prompt,
            messages: Vec::new(),
            stream: stream,
        }
    }

    // update the user message
    pub fn update_user_message(&mut self, content: String) {
        let message = ChatMessage {
            role: "user".to_string(),
            content,
            response_id: "".to_string(),
        };
        self.messages.push(message);
    }

    // update the assistant message
    pub fn update_assistant_message(&mut self, content: String, response_id: String) {
        let message = ChatMessage {
            role: "assistant".to_string(),
            content,
            response_id,
        };
        self.messages.push(message);
    }
}
