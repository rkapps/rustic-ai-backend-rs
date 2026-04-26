use agentic_core::client::{llm::CompletionStreamResponse, message::Message, response::CompletionResponseContent};
use anyhow::Result;
use rustic_ai_domain::{
    chats::{chat::Chat, config::ChatConfig},
    dto::chat::{ChatRequest, ChatResponse},
};
use std::sync::Arc;
use tracing::{debug, info};

use rustic_ai_storage::service::StorageService;

use crate::rustic::RusticService;

pub struct ChatsService {
    pub storage_service: Arc<dyn StorageService>,
    pub rustic_service: RusticService,
}

impl ChatsService {
    pub fn new(
        storage_service: Arc<dyn StorageService>,
        rustic_service: RusticService,
    ) -> ChatsService {
        ChatsService {
            storage_service,
            rustic_service,
        }
    }

    pub async fn chat_completion(&self, uid: String, request: ChatRequest) -> Result<ChatResponse> {

        let mut chat = self
            .storage_service
            .get_chat(uid, request.clone().id)
            .await?;

        let id = chat.clone().id;
        chat.update_user_message(request.prompt);
        let messages = self.create_completion_messages(&chat);

        let agent = match self
            .rustic_service
            .get_chat_agent(&chat.llm, &chat.model.to_lowercase())
        {
            Ok(c) => c,
            Err(_) => {
                return Err(anyhow::anyhow!(format!("Chat agent error")));
            }
        };
        // On error, Send an error response
        let cresponse = agent
            .complete(&chat.system, &messages)
            .await
            .map_err(|e| anyhow::anyhow!(format!("{:?}", e)))?;

        debug!("Completion Response: {:#?}", cresponse);
        //Create the chat response message and add it the chat.
        let mut rcontent = String::new();
        let response = cresponse.clone();
        for content in response.contents {
            if let CompletionResponseContent::Text(val) = content {
                // println!("The text is: {}", val);
                rcontent = val.to_string();
                chat.update_assistant_message(val.to_string(), response.response_id.clone());
            }
        }

        self.storage_service.save_chat(chat).await?;

        //Create the chat response
        let response = ChatResponse {
            id: id,
            role: "assistant".to_string(),
            content: Some(rcontent),
            response_id: Some(response.response_id),
        };
        debug!("Chat Request: {:#?}", response);

        Ok(response)
    }

    pub async fn chat_completion_streaming(
        &self,
        uid: String,
        request: ChatRequest,
    ) -> Result<CompletionStreamResponse> {

        info!("Chat request: {:?}", request.id);

        let mut chat = self
            .storage_service
            .get_chat(uid, request.clone().id)
            .await?;
        debug!("Chat: {:?}-{:?}", chat.llm, chat.model.to_lowercase());

        chat.update_user_message(request.prompt);
        let messages = self.create_completion_messages(&chat);

        let agent = match self
            .rustic_service
            .get_chat_agent(&chat.llm, &chat.model.to_lowercase())
        {
            Ok(c) => c,
            Err(_) => {
                return Err(anyhow::anyhow!(format!("Chat agent error")));
            }
        };
        debug!("Chat: {:?}", agent.llm);
        let stream = agent.complete_with_stream(&chat.system, &messages).await?;
        Ok(stream)
    }

    pub async fn create_chat(&self, uid: String, config: ChatConfig) -> Result<Chat> {
        let mut chat = config.validate().map_err(|e| anyhow::anyhow!(e))?;
        chat.uid = uid;
        let _ = self
            .storage_service
            .create_chat(chat.clone())
            .await
            .map_err(|e| anyhow::anyhow!(format!("Create Chat error: {}", e)))?;

        Ok(chat)
    }

    pub async fn delete_chat(&self, uid: String, id: String) -> Result<()> {
        self
            .storage_service
            .delete_chat(uid, id)
            .await
            .map_err(|e| anyhow::anyhow!(format!("Get Chat error: {}", e)))
        // Ok(())
    }

    pub async fn get_all_chats(&self, uid: String) -> Result<Vec<Chat>> {
        let chats = self
            .storage_service
            .get_all_chats(uid)
            .await
            .map_err(|e| anyhow::anyhow!(format!("Get All Chats error: {}", e)))?;

        Ok(chats)
    }

    pub async fn get_chat(&self, uid: String, id: String) -> Result<Chat> {
        let chat = self
            .storage_service
            .get_chat(uid, id)
            .await
            .map_err(|e| anyhow::anyhow!(format!("Get Chat error: {}", e)))?;

        Ok(chat)
    }

    fn create_completion_messages(&self, chat: &Chat) -> Vec<Message> {
        let mut nmessages = Vec::new();
        for message in chat.clone().messages {
            if message.role == "user".to_string() {
                let nmessage = Message::User {
                    content: message.content,
                    response_id: Some(message.response_id),
                };
                nmessages.push(nmessage);
            } else {
                let nmessage = Message::Assistant {
                    content: message.content,
                    response_id: Some(message.response_id),
                };
                nmessages.push(nmessage);
            }
        }
        nmessages
    }

    pub async fn save_streaming_message(
        &self,
        uid: String,
        id: &str,
        prompt: &str,
        final_content: &str,
        response_id: &str,
    ) -> Result<()> {
        let mut chat = self.get_chat(uid, id.to_string()).await?;

        chat.update_user_message(prompt.to_string());
        chat.update_assistant_message(final_content.to_string(), response_id.to_string());

        self.storage_service.save_chat(chat).await?;

        Ok(())
    }
}
