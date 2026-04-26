use anyhow::Result;
use rustic_ai_domain::chats::{CHAT_COLLECTION_NAME, chat::Chat};
use std::sync::Arc;
use storage_core::mongo::{database::MongoDatabase, repository::MongoRepository};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct MongoStorageManager {
    db: MongoDatabase,
}

impl MongoStorageManager {
    pub async fn new(uri: &str, name: &str) -> Result<Self> {
        let mut mdb = MongoDatabase::new(uri, name).await?;

        mdb.register_collection::<String, Chat>(CHAT_COLLECTION_NAME.to_string())
            .await?;

        Ok(MongoStorageManager { db: mdb })
    }

    pub async fn chats(&self) -> Result<Arc<Mutex<MongoRepository<String, Chat>>>> {
        self.db
            .collection::<String, Chat>(CHAT_COLLECTION_NAME.to_string())
            .await
    }
}
