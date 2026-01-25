//! AWS S3 + DynamoDB cache implementation for production.

use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_s3::primitives::ByteStream;

use crate::cache::{CacheError, CachedFiles, ModelCache};

/// AWS cache implementation using S3 for file storage and DynamoDB for lookup.
pub struct AwsCache {
    s3_client: aws_sdk_s3::Client,
    dynamo_client: aws_sdk_dynamodb::Client,
    bucket: String,
    table: String,
}

impl AwsCache {
    /// Create a new AwsCache with the given AWS clients and resource names.
    pub fn new(
        s3_client: aws_sdk_s3::Client,
        dynamo_client: aws_sdk_dynamodb::Client,
        bucket: String,
        table: String,
    ) -> Self {
        Self {
            s3_client,
            dynamo_client,
            bucket,
            table,
        }
    }

    /// Create a new AwsCache from environment variables.
    /// Requires S3_BUCKET_NAME and DYNAMODB_TABLE to be set.
    pub async fn from_env() -> Result<Self, CacheError> {
        let bucket = std::env::var("S3_BUCKET_NAME")
            .map_err(|_| CacheError::AwsError("S3_BUCKET_NAME not set".to_string()))?;
        let table = std::env::var("DYNAMODB_TABLE")
            .map_err(|_| CacheError::AwsError("DYNAMODB_TABLE not set".to_string()))?;

        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let s3_client = aws_sdk_s3::Client::new(&config);
        let dynamo_client = aws_sdk_dynamodb::Client::new(&config);

        Ok(Self::new(s3_client, dynamo_client, bucket, table))
    }

    fn step_key(&self, cache_key: &str) -> String {
        format!("{}/model.step", cache_key)
    }

    fn gltf_key(&self, cache_key: &str) -> String {
        format!("{}/model.gltf", cache_key)
    }
}

#[async_trait]
impl ModelCache for AwsCache {
    async fn exists(&self, cache_key: &str) -> bool {
        let result = self
            .dynamo_client
            .get_item()
            .table_name(&self.table)
            .key("plate_hash", AttributeValue::S(cache_key.to_string()))
            .send()
            .await;

        match result {
            Ok(output) => output.item.is_some(),
            Err(e) => {
                tracing::warn!("DynamoDB lookup error: {}", e);
                false
            }
        }
    }

    async fn get(&self, cache_key: &str) -> Result<CachedFiles, CacheError> {
        // Check DynamoDB first
        let dynamo_result = self
            .dynamo_client
            .get_item()
            .table_name(&self.table)
            .key("plate_hash", AttributeValue::S(cache_key.to_string()))
            .send()
            .await
            .map_err(|e| CacheError::AwsError(e.to_string()))?;

        if dynamo_result.item.is_none() {
            return Err(CacheError::NotFound);
        }

        // Fetch STEP file from S3
        let step_result = self
            .s3_client
            .get_object()
            .bucket(&self.bucket)
            .key(self.step_key(cache_key))
            .send()
            .await
            .map_err(|e| CacheError::AwsError(e.to_string()))?;

        let step_data = step_result
            .body
            .collect()
            .await
            .map_err(|e| CacheError::AwsError(e.to_string()))?
            .into_bytes()
            .to_vec();

        // Fetch glTF file from S3
        let gltf_result = self
            .s3_client
            .get_object()
            .bucket(&self.bucket)
            .key(self.gltf_key(cache_key))
            .send()
            .await
            .map_err(|e| CacheError::AwsError(e.to_string()))?;

        let gltf_data = gltf_result
            .body
            .collect()
            .await
            .map_err(|e| CacheError::AwsError(e.to_string()))?
            .into_bytes()
            .to_vec();

        tracing::info!("Cache hit for key: {}", cache_key);

        Ok(CachedFiles {
            step_data,
            gltf_data,
        })
    }

    async fn put(&self, cache_key: &str, files: &CachedFiles) -> Result<(), CacheError> {
        // Upload STEP file to S3
        self.s3_client
            .put_object()
            .bucket(&self.bucket)
            .key(self.step_key(cache_key))
            .body(ByteStream::from(files.step_data.clone()))
            .content_type("application/STEP")
            .send()
            .await
            .map_err(|e| CacheError::AwsError(e.to_string()))?;

        // Upload glTF file to S3
        self.s3_client
            .put_object()
            .bucket(&self.bucket)
            .key(self.gltf_key(cache_key))
            .body(ByteStream::from(files.gltf_data.clone()))
            .content_type("model/gltf+json")
            .send()
            .await
            .map_err(|e| CacheError::AwsError(e.to_string()))?;

        // Record in DynamoDB
        let now = chrono::Utc::now().to_rfc3339();
        self.dynamo_client
            .put_item()
            .table_name(&self.table)
            .item("plate_hash", AttributeValue::S(cache_key.to_string()))
            .item("created_at", AttributeValue::S(now))
            .send()
            .await
            .map_err(|e| CacheError::AwsError(e.to_string()))?;

        tracing::info!("Cached files for key: {}", cache_key);

        Ok(())
    }
}
