//! Local filesystem cache implementation for development.

use async_trait::async_trait;
use std::path::PathBuf;

use crate::cache::{CacheError, CachedFiles, ModelCache};

/// Local filesystem cache implementation.
/// Stores files in a directory structure: `{base_dir}/{cache_key}/model.step` and `model.gltf`.
pub struct LocalCache {
    base_dir: PathBuf,
}

impl LocalCache {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Create a LocalCache with the default cache directory ("./cache").
    pub fn default_dir() -> Self {
        Self::new(PathBuf::from("./cache"))
    }

    fn cache_dir(&self, cache_key: &str) -> PathBuf {
        self.base_dir.join(cache_key)
    }

    fn step_path(&self, cache_key: &str) -> PathBuf {
        self.cache_dir(cache_key).join("model.step")
    }

    fn gltf_path(&self, cache_key: &str) -> PathBuf {
        self.cache_dir(cache_key).join("model.gltf")
    }
}

#[async_trait]
impl ModelCache for LocalCache {
    async fn exists(&self, cache_key: &str) -> bool {
        let step_path = self.step_path(cache_key);
        let gltf_path = self.gltf_path(cache_key);
        tokio::fs::try_exists(&step_path).await.unwrap_or(false)
            && tokio::fs::try_exists(&gltf_path).await.unwrap_or(false)
    }

    async fn get(&self, cache_key: &str) -> Result<CachedFiles, CacheError> {
        let step_path = self.step_path(cache_key);
        let gltf_path = self.gltf_path(cache_key);

        let step_data = tokio::fs::read(&step_path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CacheError::NotFound
                } else {
                    CacheError::IoError(e.to_string())
                }
            })?;

        let gltf_data = tokio::fs::read(&gltf_path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CacheError::NotFound
                } else {
                    CacheError::IoError(e.to_string())
                }
            })?;

        Ok(CachedFiles {
            step_data,
            gltf_data,
        })
    }

    async fn put(&self, cache_key: &str, files: &CachedFiles) -> Result<(), CacheError> {
        let cache_dir = self.cache_dir(cache_key);

        // Create the cache directory if it doesn't exist
        tokio::fs::create_dir_all(&cache_dir)
            .await
            .map_err(|e| CacheError::IoError(e.to_string()))?;

        let step_path = self.step_path(cache_key);
        let gltf_path = self.gltf_path(cache_key);

        tokio::fs::write(&step_path, &files.step_data)
            .await
            .map_err(|e| CacheError::IoError(e.to_string()))?;

        tokio::fs::write(&gltf_path, &files.gltf_data)
            .await
            .map_err(|e| CacheError::IoError(e.to_string()))?;

        tracing::info!("Cached files for key: {}", cache_key);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_cache_put_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let cache = LocalCache::new(temp_dir.path().to_path_buf());

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
    async fn test_local_cache_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let cache = LocalCache::new(temp_dir.path().to_path_buf());

        let result = cache.get("nonexistent").await;
        assert!(matches!(result, Err(CacheError::NotFound)));
    }

    #[tokio::test]
    async fn test_local_cache_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let cache = LocalCache::new(temp_dir.path().to_path_buf());

        let files = CachedFiles {
            step_data: b"step".to_vec(),
            gltf_data: b"gltf".to_vec(),
        };

        cache.put("plate-abc123", &files).await.unwrap();

        // Verify the directory structure was created
        assert!(temp_dir.path().join("plate-abc123").exists());
        assert!(temp_dir.path().join("plate-abc123/model.step").exists());
        assert!(temp_dir.path().join("plate-abc123/model.gltf").exists());
    }
}
