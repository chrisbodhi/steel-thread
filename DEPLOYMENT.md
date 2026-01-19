# Platerator AWS Deployment Guide

## Quick Start (Using Just)

All deployment commands are available via `just`:

```bash
# See all available commands
just

# Initial setup
just tf-init              # Initialize Terraform
just tf-plan              # Plan infrastructure (review changes)
just tf-apply             # Create AWS infrastructure (requires approval)
just set-zoo-token        # Store zoo token in AWS Secrets Manager

# Deploy application
just deploy               # Download latest build, deploy to Lightsail

# Monitoring
just status               # Check Lightsail service status
just logs                 # Tail service logs
just ssh                  # SSH to Lightsail instance
just info                 # Show all terraform outputs (IP, URLs)

# Token Management
just deploy-zoo-token     # Deploy zoo token to instance (after rotation)

# Cleanup
just tf-destroy           # Destroy all AWS infrastructure
```

## Overview

Platerator is deployed on AWS Lightsail (nano instance, $3.50/month) with S3 storage and DynamoDB caching. No Docker required—GitHub Actions builds the Linux binary, you deploy to Lightsail using your local AWS credentials.

## Architecture

```
User → Lightsail (Rust/Axum via systemd) → DynamoDB (cache lookup)
                                            ↓ (cache miss)
                                       Generate Model (zoo CLI)
                                            ↓
                                       Upload to S3
                                            ↓
                                       Store hash → S3 key in DynamoDB
                                            ↓
                                       Return pre-signed URLs
```

## Cost Estimate

| Service | Monthly Cost (est.) |
|---------|---------------------|
| Lightsail nano (Ubuntu) | **$3.50** |
| S3 (50GB + transfer) | ~$1-2 |
| DynamoDB (on-demand) | ~$0.50 |
| Secrets Manager | $0.40 |
| CloudWatch Logs | ~$0.50 |
| **Total** | **~$6-7/month** |

With AWS free tier (first year): **~$3-4/month**

## Prerequisites

- AWS Account
- AWS CLI configured (`aws configure`)
- Terraform >= 1.2
- GitHub CLI (`gh`) or SSH key for artifact download
- zoo CLI token (from https://zoo.dev)

## Deployment Steps

### 1. Set up zoo CLI token file

Create `~/.config/zoo/hosts.toml`:

```toml
["https://api.zoo.dev/"]
token = "dev-xxx-your-token"
user = "your@email.com"
```

### 2. Initialize Terraform

```bash
# From project root
just tf-init

# This initializes Terraform and creates the plan file
```

### 3. Plan infrastructure (review before applying)

```bash
just tf-plan
```

Review the output carefully. You should see resources being created for:
- Lightsail instance (Ubuntu 22.04 nano)
- Static IP address
- S3 bucket
- DynamoDB table
- Secrets Manager secret
- CloudWatch log group + dashboard

### 4. Apply infrastructure (**DO NOT RUN WITHOUT APPROVAL**)

**STOP**: Get approval before running this command.

```bash
just tf-apply
```

Type `yes` when prompted. The instance will take ~5 minutes to finish setup (installing Rust, zoo CLI, Caddy, systemd service).

### 5. Store zoo token in Secrets Manager

```bash
just set-zoo-token
```

This reads your local zoo token from `~/.config/zoo/hosts.toml` and stores it in AWS Secrets Manager for the instance to use.

### 6. Wait for instance setup to complete

```bash
# Get the public IP
LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip)

# Watch the setup progress (connect as ubuntu user)
ssh ubuntu@$LIGHTSAIL_IP 'tail -f /var/log/cloud-init-output.log'
```

Wait for the message: **"Lightsail instance setup complete!"**

Then exit with `Ctrl+C`.

### 7. Push code to GitHub (triggers build)

```bash
# Make sure code is committed
git add .
git commit -m "Deploy to production"
git push origin master

# This triggers GitHub Actions to build the Linux binary
# Check progress at: https://github.com/yourusername/steel-thread/actions
```

Wait for the build to complete (~5 minutes).

### 8. Download and deploy to Lightsail

```bash
# Download the build artifact and deploy to your Lightsail instance
just deploy
```

This will:
1. Download the latest successful build from GitHub Actions
2. SCP the tarball to your Lightsail instance
3. Extract and install the app
4. Restart the systemd service
5. Verify it's running

### 9. Get your app URL

```bash
just info
```

Look for `service_url` in the output. Visit that URL to see Platerator running!

Example: `http://123.45.67.89`

## AWS Integration Implementation

The current codebase uses in-memory session storage. The infrastructure is ready (S3, DynamoDB) but the application integration is still a TODO. Here's how to implement it:

### Add AWS client module (`crates/web/src/aws.rs`)

```rust
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::presigning::PresigningConfig;
use domain::ActuatorPlate;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;

#[derive(Clone)]
pub struct AwsClients {
    pub s3: S3Client,
    pub dynamodb: DynamoDbClient,
    pub bucket_name: String,
    pub table_name: String,
    pub region: String,
}

impl AwsClients {
    pub async fn new() -> Self {
        let config = aws_config::load_from_env().await;
        let bucket_name = std::env::var("S3_BUCKET_NAME")
            .expect("S3_BUCKET_NAME must be set");
        let table_name = std::env::var("DYNAMODB_TABLE")
            .expect("DYNAMODB_TABLE must be set");
        let region = std::env::var("AWS_REGION")
            .unwrap_or_else(|_| "us-east-1".to_string());

        Self {
            s3: S3Client::new(&config),
            dynamodb: DynamoDbClient::new(&config),
            bucket_name,
            table_name,
            region,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    pub plate_hash: String,
    pub step_s3_key: String,
    pub gltf_s3_key: String,
    pub parameters: ActuatorPlate,
    pub created_at: String,
    pub access_count: i64,
    pub last_accessed: String,
    pub ttl: i64,
}

/// Compute deterministic hash of plate parameters
pub fn compute_plate_hash(plate: &ActuatorPlate) -> String {
    let mut hasher = Sha256::new();

    // Hash fields in sorted order for consistency
    hasher.update(format!("bolt_diameter:{}", plate.bolt_diameter.0));
    hasher.update(format!("bolt_spacing:{}", plate.bolt_spacing.0));
    hasher.update(format!("bracket_height:{}", plate.bracket_height.0));
    hasher.update(format!("bracket_width:{}", plate.bracket_width.0));
    hasher.update(format!("pin_count:{}", plate.pin_count));
    hasher.update(format!("pin_diameter:{}", plate.pin_diameter.0));
    hasher.update(format!("plate_thickness:{}", plate.plate_thickness.0));

    format!("{:x}", hasher.finalize())
}

/// Check DynamoDB cache for existing generation
pub async fn check_cache(
    clients: &AwsClients,
    plate_hash: &str,
) -> Option<CacheEntry> {
    use aws_sdk_dynamodb::types::AttributeValue;

    let result = clients.dynamodb
        .get_item()
        .table_name(&clients.table_name)
        .key("plate_hash", AttributeValue::S(plate_hash.to_string()))
        .send()
        .await
        .ok()?;

    let item = result.item?;

    // Parse cache entry
    Some(CacheEntry {
        plate_hash: item.get("plate_hash")?.as_s().ok()?.clone(),
        step_s3_key: item.get("step_s3_key")?.as_s().ok()?.clone(),
        gltf_s3_key: item.get("gltf_s3_key")?.as_s().ok()?.clone(),
        parameters: serde_json::from_str(
            item.get("parameters")?.as_s().ok()?
        ).ok()?,
        created_at: item.get("created_at")?.as_s().ok()?.clone(),
        access_count: item.get("access_count")?.as_n().ok()?.parse().ok()?,
        last_accessed: item.get("last_accessed")?.as_s().ok()?.clone(),
        ttl: item.get("ttl")?.as_n().ok()?.parse().ok()?,
    })
}

/// Upload file to S3
pub async fn upload_to_s3(
    clients: &AwsClients,
    key: &str,
    file_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = aws_sdk_s3::primitives::ByteStream::from_path(file_path).await?;

    clients.s3
        .put_object()
        .bucket(&clients.bucket_name)
        .key(key)
        .body(body)
        .send()
        .await?;

    Ok(())
}

/// Generate pre-signed URL for S3 object (1 hour expiry)
pub async fn generate_presigned_url(
    clients: &AwsClients,
    key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let presigning_config = PresigningConfig::expires_in(Duration::from_secs(3600))?;

    let presigned_req = clients.s3
        .get_object()
        .bucket(&clients.bucket_name)
        .key(key)
        .presigned(presigning_config)
        .await?;

    Ok(presigned_req.uri().to_string())
}

/// Store cache entry in DynamoDB
pub async fn store_cache(
    clients: &AwsClients,
    entry: &CacheEntry,
) -> Result<(), Box<dyn std::error::Error>> {
    use aws_sdk_dynamodb::types::AttributeValue;

    clients.dynamodb
        .put_item()
        .table_name(&clients.table_name)
        .item("plate_hash", AttributeValue::S(entry.plate_hash.clone()))
        .item("step_s3_key", AttributeValue::S(entry.step_s3_key.clone()))
        .item("gltf_s3_key", AttributeValue::S(entry.gltf_s3_key.clone()))
        .item("parameters", AttributeValue::S(
            serde_json::to_string(&entry.parameters)?
        ))
        .item("created_at", AttributeValue::S(entry.created_at.clone()))
        .item("access_count", AttributeValue::N(entry.access_count.to_string()))
        .item("last_accessed", AttributeValue::S(entry.last_accessed.clone()))
        .item("ttl", AttributeValue::N(entry.ttl.to_string()))
        .send()
        .await?;

    Ok(())
}
```

### Update `crates/web/src/lib.rs` to use AWS

```rust
// Add at top of file
mod aws;
use aws::{AwsClients, check_cache, compute_plate_hash, generate_presigned_url, store_cache, upload_to_s3, CacheEntry};

// Update AppState to include AWS clients
pub struct AppStateInner {
    sessions: HashMap<String, GenerationResult>,
    aws_clients: Option<AwsClients>,
}

pub type AppState = Arc<RwLock<AppStateInner>>;

// Update run() function
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Initialize AWS clients if environment variables are set
    let aws_clients = if std::env::var("S3_BUCKET_NAME").is_ok() {
        Some(AwsClients::new().await)
    } else {
        tracing::warn!("AWS environment variables not set, using in-memory storage");
        None
    };

    let state: AppState = Arc::new(RwLock::new(AppStateInner {
        sessions: HashMap::new(),
        aws_clients,
    }));

    // ... rest of function
}

// Update generate_plate_model to use AWS caching
pub async fn generate_plate_model(
    State(state): State<AppState>,
    Json(payload): Json<ActuatorPlate>,
) -> impl IntoResponse {
    let state_lock = state.read().await;

    // If AWS is configured, use S3 + DynamoDB caching
    if let Some(ref aws_clients) = state_lock.aws_clients {
        drop(state_lock); // Release lock before async operations

        let plate_hash = compute_plate_hash(&payload);

        // Check cache
        if let Some(cache_entry) = check_cache(aws_clients, &plate_hash).await {
            tracing::info!("Cache hit for plate_hash: {}", plate_hash);

            // Generate pre-signed URLs
            match (
                generate_presigned_url(aws_clients, &cache_entry.step_s3_key).await,
                generate_presigned_url(aws_clients, &cache_entry.gltf_s3_key).await,
            ) {
                (Ok(step_url), Ok(gltf_url)) => {
                    let res = GenerateSuccessResponse {
                        success: true,
                        message: "Model retrieved from cache".to_string(),
                        download_url: step_url,
                        gltf_url,
                        session_id: plate_hash,
                    };
                    return (StatusCode::OK, Json(res)).into_response();
                }
                _ => {
                    tracing::error!("Failed to generate pre-signed URLs");
                }
            }
        }

        // Cache miss - generate model
        tracing::info!("Cache miss for plate_hash: {}, generating model", plate_hash);

        let result = match generate_model(&payload) {
            Ok(r) => r,
            Err(e) => {
                // ... error handling
                return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
            }
        };

        // Upload to S3
        let now = chrono::Utc::now();
        let step_key = format!(
            "generated/{}/{}/output.step",
            now.format("%Y-%m"),
            &plate_hash[..8]
        );
        let gltf_key = format!(
            "generated/{}/{}/source.gltf",
            now.format("%Y-%m"),
            &plate_hash[..8]
        );

        if let Err(e) = upload_to_s3(aws_clients, &step_key, &result.step_file).await {
            tracing::error!("Failed to upload STEP to S3: {}", e);
        }

        if let Err(e) = upload_to_s3(aws_clients, &gltf_key, &result.gltf_file).await {
            tracing::error!("Failed to upload glTF to S3: {}", e);
        }

        // Store in cache (90 day TTL)
        let cache_entry = CacheEntry {
            plate_hash: plate_hash.clone(),
            step_s3_key: step_key.clone(),
            gltf_s3_key: gltf_key.clone(),
            parameters: payload.clone(),
            created_at: now.to_rfc3339(),
            access_count: 1,
            last_accessed: now.to_rfc3339(),
            ttl: (now.timestamp() + 90 * 24 * 3600),
        };

        if let Err(e) = store_cache(aws_clients, &cache_entry).await {
            tracing::error!("Failed to store cache entry: {}", e);
        }

        // Generate pre-signed URLs
        match (
            generate_presigned_url(aws_clients, &step_key).await,
            generate_presigned_url(aws_clients, &gltf_key).await,
        ) {
            (Ok(step_url), Ok(gltf_url)) => {
                let res = GenerateSuccessResponse {
                    success: true,
                    message: "Model generated and uploaded to S3".to_string(),
                    download_url: step_url,
                    gltf_url,
                    session_id: plate_hash,
                };
                (StatusCode::OK, Json(res)).into_response()
            }
            _ => {
                let res = ErrorResponse {
                    success: false,
                    got_it: false,
                    errors: vec!["Failed to generate download URLs".to_string()],
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(res)).into_response()
            }
        }
    } else {
        // Fallback to in-memory storage (local development)
        drop(state_lock);
        // ... existing in-memory implementation
    }
}
```

### Add chrono dependency

```toml
# crates/web/Cargo.toml
chrono = "0.4"
```

## GitHub Actions CI/CD

GitHub Actions automatically builds the Linux binary when you push to the main branch. The build artifacts (tarball with compiled binary + frontend) are stored as release artifacts for you to download and deploy locally.

**Build Configuration**:
- Builds on Ubuntu 22.04 (for GLIBC 2.35 compatibility with Lightsail instance)
- Runs tests and clippy linting before building
- Packages binary + frontend dist + KCL files into tarball
- Artifacts retained for 30 days

**No automatic deployment to production**—credentials stay on your machine using `just deploy`.

**Deployment Safety**: The deployment script will error if multiple tarballs exist on the server, preventing accidental deployment of the wrong version.

## Monitoring

### Lightsail Instance Logs

View logs from your instance:

```bash
# SSH to instance and view systemd logs
just ssh
journalctl -u platerator -f

# Or from your machine
LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip)
ssh ubuntu@$LIGHTSAIL_IP 'sudo journalctl -u platerator -n 100'
```

### CloudWatch Dashboard

View metrics at: https://console.aws.amazon.com/cloudwatch/home?region=us-east-1#dashboards:

Dashboard includes:
- Request logs from the instance
- DynamoDB capacity usage
- S3 storage size

### Service Status

```bash
# Check if the app is running
just status

# Or manually check
LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip)
ssh ubuntu@$LIGHTSAIL_IP 'systemctl status platerator'
```

## Troubleshooting

### Deployment fails with "Service won't start"

1. SSH to instance and check systemd status:
```bash
just ssh
systemctl status platerator
journalctl -u platerator -n 50
```

2. Check if the binary was downloaded correctly:
```bash
ls -lh /opt/platerator/
```

3. Verify zoo token was stored and deployed:
```bash
# Check Secrets Manager
aws secretsmanager get-secret-value --secret-id platerator/zoo-token --region us-east-1

# Check it's on the instance
ssh ubuntu@YOUR_IP 'sudo cat /opt/platerator/.env'
```

### Instance is unreachable (SSH fails)

The instance may still be setting up. Wait 5 minutes after `terraform apply`:

```bash
LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip)
ssh ubuntu@$LIGHTSAIL_IP 'tail -f /var/log/cloud-init-output.log'
```

### Files not caching

1. Check DynamoDB table:
```bash
aws dynamodb scan --table-name platerator-cache --max-items 5
```

2. Check S3 bucket:
```bash
aws s3 ls s3://platerator-generated-files-us-east-1/generated/ --recursive
```

3. Check instance logs for cache-related errors:
```bash
just logs | grep -i cache
```

### High AWS costs

1. Check S3 storage size:
```bash
aws s3 ls s3://platerator-generated-files-us-east-1/ --recursive --summarize
```

2. Check if S3 lifecycle policies are working (should transition to Glacier after 90 days):
```bash
aws s3api get-bucket-lifecycle-configuration --bucket platerator-generated-files-us-east-1
```

3. Check Lightsail bandwidth usage:
```bash
LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip)
ssh ubuntu@$LIGHTSAIL_IP 'vnstat'  # If installed, shows network stats
```

## Destroying Infrastructure

**WARNING**: This will delete ALL data including S3 files and the Lightsail instance!

```bash
# Remove S3 bucket contents first (terraform can't delete non-empty buckets)
BUCKET_NAME=$(cd terraform && terraform output -raw s3_bucket_name)
aws s3 rm s3://$BUCKET_NAME/ --recursive

# Destroy infrastructure
just tf-destroy
```

Type `yes` when prompted.

## Next Steps

- [ ] Implement AWS S3/DynamoDB integration code (see above)
- [ ] Set up custom domain (point DNS to Lightsail static IP, update Caddy config)
- [ ] Add monitoring and alerting (optional)
- [ ] Configure auto-scaling (upgrade to larger Lightsail bundle if needed)
