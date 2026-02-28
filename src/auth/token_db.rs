use std::collections::HashMap;

use tokio::sync::RwLock;

/// The database storing all the access token
#[derive(Debug, Default)]
pub struct AccessTokenDatabase {
    tokens: RwLock<HashMap<String, String>>,
}

impl AccessTokenDatabase {
    /// Store a new token in the database
    pub async fn register(&self, token: String, id: String) {
        self.tokens.write().await.insert(token, id);
    }

    /// Return the id of the matching token if there is one
    pub fn get_blocking(&self, token: String) -> Option<String> {
        self.tokens.blocking_read().get(&token).cloned()
    }
}
