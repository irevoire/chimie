use std::collections::HashMap;

use tokio::sync::RwLock;
use uuid::Uuid;

/// The database storing all the access token
#[derive(Debug)]
pub struct AccessTokenDatabase {
    tokens: RwLock<HashMap<Uuid, String>>,
}

impl AccessTokenDatabase {
    pub async fn register(&self, token: Uuid, id: String) {
        self.tokens.write().await.insert(token, id);
    }
}
