//! Cache trait and types for storing generated model files.

use async_trait::async_trait;
use std::fmt;

/// Cached model files containing STEP and glTF data.
#[derive(Clone)]
pub struct CachedFiles {
    pub step_data: Vec<u8>,
    pub gltf_data: Vec<u8>,
}

/// Errors that can occur during cache operations.
#[derive(Debug)]
pub enum CacheError {
    /// The requested cache key was not found.
    NotFound,
    /// An I/O error occurred during cache operations.
    IoError(String),
    /// An AWS service error occurred.
    AwsError(String),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::NotFound => write!(f, "Cache entry not found"),
            CacheError::IoError(msg) => write!(f, "Cache I/O error: {}", msg),
            CacheError::AwsError(msg) => write!(f, "AWS error: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

/// Trait for caching generated model files.
///
/// Implementations store and retrieve STEP and glTF files using a deterministic
/// cache key derived from the plate configuration.
#[async_trait]
pub trait ModelCache: Send + Sync {
    /// Check if a cache entry exists for the given key.
    async fn exists(&self, cache_key: &str) -> bool;

    /// Retrieve cached files for the given key.
    async fn get(&self, cache_key: &str) -> Result<CachedFiles, CacheError>;

    /// Store files in the cache with the given key.
    async fn put(&self, cache_key: &str, files: &CachedFiles) -> Result<(), CacheError>;
}
