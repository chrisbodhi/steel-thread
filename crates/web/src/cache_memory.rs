//! In-memory cache implementation for testing.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::cache::{CacheError, CachedFiles, ModelCache};

/// In-memory cache implementation using a HashMap.
/// Useful for testing and development without filesystem dependencies.
pub struct MemoryCache {
    entries: RwLock<HashMap<String, CachedFiles>>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModelCache for MemoryCache {
    async fn exists(&self, cache_key: &str) -> bool {
        let entries = self.entries.read().unwrap();
        entries.contains_key(cache_key)
    }

    async fn get(&self, cache_key: &str) -> Result<CachedFiles, CacheError> {
        let entries = self.entries.read().unwrap();
        entries
            .get(cache_key)
            .cloned()
            .ok_or(CacheError::NotFound)
    }

    async fn put(&self, cache_key: &str, files: &CachedFiles) -> Result<(), CacheError> {
        let mut entries = self.entries.write().unwrap();
        entries.insert(cache_key.to_string(), files.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache_put_and_get() {
        let cache = MemoryCache::new();
        let files = CachedFiles {
            step_data: b"step content".to_vec(),
            gltf_data: b"gltf content".to_vec(),
        };

        assert!(!cache.exists("test-key").await);

        cache.put("test-key", &files).await.unwrap();

        assert!(cache.exists("test-key").await);

        let retrieved = cache.get("test-key").await.unwrap();
        assert_eq!(retrieved.step_data, files.step_data);
        assert_eq!(retrieved.gltf_data, files.gltf_data);
    }

    #[tokio::test]
    async fn test_memory_cache_not_found() {
        let cache = MemoryCache::new();
        let result = cache.get("nonexistent").await;
        assert!(matches!(result, Err(CacheError::NotFound)));
    }
}
