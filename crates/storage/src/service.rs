use anyhow::Result;
use rustic_ai_domain::chats::chat::Chat;
use std::fmt::Debug;

use async_trait::async_trait;

#[async_trait]
pub trait StorageService: ChatStorageService + Send + Sync + Debug {}

#[async_trait]
pub trait ChatStorageService: Send + Sync + Debug {
    async fn create_chat(&self, chat: Chat) -> Result<()>;
    async fn delete_chat(&self, uid: String, id: String) -> Result<()>;
    async fn get_all_chats(&self, uid: String) -> Result<Vec<Chat>>;
    async fn get_chat(&self, uid: String, id: String) -> Result<Chat>;
    async fn save_chat(&self, chat: Chat) -> Result<()>;
}
