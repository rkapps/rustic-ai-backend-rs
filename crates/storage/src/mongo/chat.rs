use crate::{mongo::service::MongoStorageService, service::ChatStorageService};
use anyhow::Result;
use async_trait::async_trait;
use rustic_ai_domain::{ID, UID, chats::chat::Chat};
use storage_core::core::{
    Repository,
    search::{SearchCriteria, SearchOp, SearchValue},
};

#[async_trait]
impl ChatStorageService for MongoStorageService {
    async fn create_chat(&self, chat: Chat) -> Result<()> {
        let Ok(repo) = self.manager.chats().await else {
            return Err(anyhow::anyhow!("Error getting Chat Repository",));
        };

        let mut repo = repo.lock().await;
        repo.insert(chat).await?;
        Ok(())
    }

    async fn delete_chat(&self, uid: String, id: String) -> Result<()> {
        let chat = self.get_chat(uid, id).await?;
        let Ok(repo) = self.manager.chats().await else {
            return Err(anyhow::anyhow!("Error getting Chat Repository",));
        };
        let mut repo = repo.lock().await;
        repo.delete(chat).await?;
        Ok(())
    }

    async fn get_all_chats(&self, uid: String) -> Result<Vec<Chat>> {
        let Ok(repo) = self.manager.chats().await else {
            return Err(anyhow::anyhow!("Error getting Chat Repository",));
        };
        let mut repo = repo.lock().await;
        let mut criteria = SearchCriteria::new();
        criteria.add_condition(UID, SearchOp::Eq, SearchValue::String(uid));
        repo.find(Some(criteria)).await
    }

    async fn get_chat(&self, uid: String, id: String) -> Result<Chat> {
        let Ok(repo) = self.manager.chats().await else {
            return Err(anyhow::anyhow!("Error getting Chat Repository",));
        };
        let mut repo = repo.lock().await;
        let mut criteria = SearchCriteria::new();
        criteria.add_condition(UID, SearchOp::Eq, SearchValue::String(uid));
        criteria.add_condition(ID, SearchOp::Eq, SearchValue::String(id));
        let chat = repo.find_one(Some(criteria)).await?;
        Ok(chat)
    }

    async fn save_chat(&self, chat: Chat) -> Result<()> {
        let Ok(repo) = self.manager.chats().await else {
            return Err(anyhow::anyhow!("Error getting Chat Repository",));
        };

        let mut repo = repo.lock().await;
        repo.update(chat).await?;
        Ok(())
    }
}
