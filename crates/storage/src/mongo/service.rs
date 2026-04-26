use std::fmt::Debug;

use crate::{
    mongo::manager::MongoStorageManager,
    service::{ChatStorageService, StorageService},
};

#[derive(Debug)]
pub struct MongoStorageService {
    pub manager: MongoStorageManager,
}
impl MongoStorageService {
    pub fn new(manager: MongoStorageManager) -> Self {
        Self { manager }
    }
}

// 2. The Blanket Implementation (The "Glue")
impl<T> StorageService for T where T: ChatStorageService + Send + Sync + Debug {}
