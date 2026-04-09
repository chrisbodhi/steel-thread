# Order Queue Architecture

This document describes the architecture for adding an order queue to Platerator. The queue enables:

1. **Server writes**: API accepts orders and enqueues them
2. **Server reads**: API retrieves queue state for display
3. **Server reorders**: API supports priority changes and reordering
4. **Printer pulls**: External printer system dequeues and processes orders
5. **Airplane mode**: Local development without AWS dependencies

## Overview

```
┌─────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Frontend  │────▶│   Platerator    │────▶│   Order Queue   │
│   (React)   │     │   API (Axum)    │     │   (DynamoDB)    │
└─────────────┘     └─────────────────┘     └────────┬────────┘
                                                     │
                                                     ▼
                                            ┌─────────────────┐
                                            │  Printer System │
                                            │  (External)     │
                                            └─────────────────┘
```

## Design Decisions

### Why DynamoDB (not SQS)?

| Requirement | SQS | DynamoDB |
|-------------|-----|----------|
| Reorder items | ❌ Not supported | ✅ Update position field |
| View full queue | ❌ Destructive reads | ✅ Query without consuming |
| Priority changes | ❌ No random access | ✅ Update any item |
| Dequeue for printer | ✅ Built-in | ✅ Conditional update |
| At-least-once delivery | ✅ Built-in | ✅ Via status field |
| Ordered retrieval | ❌ Best-effort | ✅ Sort key ordering |

**Decision**: Use DynamoDB with a position-based ordering scheme. SQS's FIFO guarantees don't help when we need to reorder items or display queue state to users.

### Queue Backend Abstraction

Following the existing `ModelCache` pattern, we'll create an `OrderQueue` trait with multiple implementations:

```rust
#[async_trait]
pub trait OrderQueue: Send + Sync {
    /// Add an order to the queue
    async fn enqueue(&self, order: Order) -> Result<Order, QueueError>;

    /// Get all pending orders, sorted by position
    async fn list_pending(&self) -> Result<Vec<Order>, QueueError>;

    /// Get a specific order by ID
    async fn get(&self, order_id: &str) -> Result<Option<Order>, QueueError>;

    /// Update order position (for reordering)
    async fn update_position(&self, order_id: &str, new_position: i64) -> Result<(), QueueError>;

    /// Claim next order for processing (atomic)
    async fn claim_next(&self) -> Result<Option<Order>, QueueError>;

    /// Mark order as completed
    async fn complete(&self, order_id: &str, result: OrderResult) -> Result<(), QueueError>;

    /// Mark order as failed
    async fn fail(&self, order_id: &str, error: &str) -> Result<(), QueueError>;
}
```

## Data Model

### Order Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Order {
    /// Unique order identifier (UUID)
    pub order_id: String,

    /// Plate configuration to manufacture
    pub plate: ActuatorPlate,

    /// Queue position (lower = higher priority)
    /// Uses fractional positioning for efficient reordering
    pub position: f64,

    /// Current order status
    pub status: OrderStatus,

    /// When the order was created
    pub created_at: DateTime<Utc>,

    /// When processing started (if claimed)
    pub claimed_at: Option<DateTime<Utc>>,

    /// When processing completed (if done)
    pub completed_at: Option<DateTime<Utc>>,

    /// Worker ID that claimed this order
    pub claimed_by: Option<String>,

    /// Result URLs after completion
    pub result: Option<OrderResult>,

    /// Error message if failed
    pub error: Option<String>,

    /// If this is a re-order, reference to the original order
    pub original_order_id: Option<String>,

    /// Whether this order is a courtesy re-order (no charge)
    pub is_courtesy_reorder: bool,

    /// Who initiated the re-order (service account ID)
    pub reordered_by: Option<String>,

    /// Reason for re-order (for audit trail)
    pub reorder_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// Waiting in queue
    Pending,
    /// Claimed by printer, being processed
    Processing,
    /// Successfully completed
    Completed,
    /// Failed with error
    Failed,
    /// Cancelled by user
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrderResult {
    /// S3 key or local path to STEP file
    pub step_file: String,
    /// S3 key or local path to glTF file
    pub gltf_file: String,
}
```

### DynamoDB Table Design

**Table Name**: `platerator-orders` (configurable via `DYNAMODB_ORDERS_TABLE` env var)

| Attribute | Type | Key | Description |
|-----------|------|-----|-------------|
| `order_id` | String | PK | UUID |
| `status` | String | GSI-PK | For querying by status |
| `position` | Number | GSI-SK | For ordered retrieval |
| `plate` | Map | - | Serialized ActuatorPlate |
| `created_at` | String | - | ISO 8601 timestamp |
| `claimed_at` | String | - | ISO 8601 timestamp |
| `completed_at` | String | - | ISO 8601 timestamp |
| `claimed_by` | String | - | Worker identifier |
| `result` | Map | - | STEP/glTF file references |
| `error` | String | - | Failure reason |
| `ttl` | Number | - | Auto-cleanup for old orders |
| `original_order_id` | String | - | Reference to original order (for re-orders) |
| `is_courtesy_reorder` | Boolean | - | True if no-charge re-order |
| `reordered_by` | String | - | Service account that initiated re-order |
| `reorder_reason` | String | - | Audit trail for re-order |

**Global Secondary Index (GSI)**: `status-position-index`
- Partition key: `status`
- Sort key: `position`
- Enables efficient queries like "get all pending orders sorted by position"

### Position Strategy: Fractional Indexing

To enable efficient reordering without updating multiple rows:

```
Initial queue:     [A: 1.0] [B: 2.0] [C: 3.0]
Insert D after A:  [A: 1.0] [D: 1.5] [B: 2.0] [C: 3.0]
Move C to front:   [C: 0.5] [A: 1.0] [D: 1.5] [B: 2.0]
```

Position calculation for inserting between items at positions `p1` and `p2`:
```rust
new_position = (p1 + p2) / 2.0
```

For inserting at the beginning (before position `p`):
```rust
new_position = p - 1.0
```

For appending at the end (after position `p`):
```rust
new_position = p + 1.0
```

**Normalization**: Periodically renumber positions (1, 2, 3, ...) when precision gets too low (positions too close together).

## API Endpoints

### New Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/orders` | Create new order |
| GET | `/api/orders` | List orders (with status filter) |
| GET | `/api/orders/{id}` | Get order details |
| PATCH | `/api/orders/{id}/position` | Reorder (change position) |
| DELETE | `/api/orders/{id}` | Cancel order |
| POST | `/api/orders/claim` | Printer claims next order |
| POST | `/api/orders/{id}/complete` | Mark order completed |
| POST | `/api/orders/{id}/fail` | Mark order failed |
| POST | `/api/orders/{id}/reorder` | **[Authenticated]** Re-order (duplicate at no cost) |

### Request/Response Examples

#### Create Order
```http
POST /api/orders
Content-Type: application/json

{
  "plate": {
    "bolt_spacing": 60,
    "bolt_size": "M10",
    "bracket_height": 40,
    "bracket_width": 30,
    "material": "aluminum",
    "pin_diameter": 10,
    "pin_count": 6,
    "plate_thickness": 8
  }
}
```

Response:
```json
{
  "order_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "pending",
  "position": 42.0,
  "created_at": "2026-01-30T12:00:00Z"
}
```

#### List Pending Orders
```http
GET /api/orders?status=pending
```

Response:
```json
{
  "orders": [
    {
      "order_id": "550e8400-...",
      "status": "pending",
      "position": 1.0,
      "plate": { ... },
      "created_at": "2026-01-30T12:00:00Z"
    },
    {
      "order_id": "660e8400-...",
      "status": "pending",
      "position": 2.0,
      "plate": { ... },
      "created_at": "2026-01-30T12:01:00Z"
    }
  ]
}
```

#### Reorder
```http
PATCH /api/orders/550e8400-.../position
Content-Type: application/json

{
  "after": "660e8400-..."  // Move after this order
}
```

Or move to front:
```json
{
  "position": "first"
}
```

#### Printer Claims Order
```http
POST /api/orders/claim
Content-Type: application/json

{
  "worker_id": "printer-01"
}
```

Response (atomic claim):
```json
{
  "order_id": "550e8400-...",
  "status": "processing",
  "plate": { ... },
  "claimed_at": "2026-01-30T12:05:00Z",
  "claimed_by": "printer-01"
}
```

Returns `204 No Content` if queue is empty.

#### Complete Order
```http
POST /api/orders/550e8400-.../complete
Content-Type: application/json

{
  "worker_id": "printer-01",
  "result": {
    "step_file": "orders/550e8400-.../model.step",
    "gltf_file": "orders/550e8400-.../model.gltf"
  }
}
```

#### Re-order (Authenticated Service Endpoint)

Creates a new order duplicating an existing order's configuration, marked as a courtesy re-order (no charge to customer). This endpoint requires service account authentication.

```http
POST /api/orders/550e8400-.../reorder
Authorization: Bearer <service-account-token>
Content-Type: application/json

{
  "reason": "Print quality issue detected by QA system",
  "priority": "high"  // Optional: "high" places at front of queue
}
```

Response:
```json
{
  "order_id": "770e8400-e29b-41d4-a716-446655440000",
  "original_order_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "pending",
  "position": 0.5,
  "is_courtesy_reorder": true,
  "reordered_by": "ai-qa-system",
  "reorder_reason": "Print quality issue detected by QA system",
  "plate": { ... },
  "created_at": "2026-01-30T14:00:00Z"
}
```

**Authentication**: Requires a valid service account token (see [Service Account Authentication](#service-account-authentication)).

**Behavior**:
- Creates a new order with the same `plate` configuration as the original
- Sets `is_courtesy_reorder: true` (excluded from billing)
- Links to original via `original_order_id` for audit trail
- Records `reordered_by` (service account ID) and `reorder_reason`
- Optional `priority: "high"` places order at front of queue

## Service Account Authentication

Certain endpoints require service account authentication for machine-to-machine access. This is used by:

- **AI QA System**: Re-orders for quality issues
- **Printer System**: Claiming and completing orders
- **Internal Tools**: Administrative operations

### Authentication Flow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   AI System     │────▶│   Platerator    │────▶│   Token Store   │
│   (Service)     │     │   API           │     │   (DynamoDB)    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        │                       │
        │  Authorization:       │  Validate token,
        │  Bearer <token>       │  check permissions
        │                       │
```

### Token Format

Service account tokens are pre-shared secrets stored in environment variables or a secrets manager:

```
PLATERATOR_SERVICE_TOKEN_AI_QA=sk_service_abc123...
PLATERATOR_SERVICE_TOKEN_PRINTER=sk_service_def456...
```

### Implementation

```rust
/// Service account identity extracted from token
#[derive(Debug, Clone)]
pub struct ServiceAccount {
    pub id: String,           // e.g., "ai-qa-system"
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    /// Can create courtesy re-orders
    ReorderCreate,
    /// Can claim orders for processing
    OrderClaim,
    /// Can mark orders complete/failed
    OrderComplete,
    /// Can view all orders (admin)
    OrderReadAll,
}

/// Axum extractor for authenticated service requests
pub struct AuthenticatedService(pub ServiceAccount);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedService
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                success: false,
                errors: vec!["Missing Authorization header".into()],
            })))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                success: false,
                errors: vec!["Invalid Authorization format".into()],
            })))?;

        // Validate token against configured service accounts
        let service = validate_service_token(token)
            .ok_or((StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                success: false,
                errors: vec!["Invalid service token".into()],
            })))?;

        Ok(AuthenticatedService(service))
    }
}
```

### Re-order Handler

```rust
#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/reorder",
    tag = "orders",
    security(("service_token" = [])),
    request_body = ReorderRequest,
    responses(
        (status = 201, description = "Re-order created", body = Order),
        (status = 401, description = "Invalid or missing service token"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Original order not found")
    )
)]
async fn reorder(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    AuthenticatedService(service): AuthenticatedService,
    Json(request): Json<ReorderRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Check permission
    if !service.permissions.contains(&Permission::ReorderCreate) {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse {
            success: false,
            errors: vec!["Service account lacks ReorderCreate permission".into()],
        })));
    }

    // Get original order
    let original = state.queue
        .get(&order_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            success: false,
            errors: vec![e.to_string()],
        })))?
        .ok_or((StatusCode::NOT_FOUND, Json(ErrorResponse {
            success: false,
            errors: vec!["Order not found".into()],
        })))?;

    // Create new order from original
    let mut new_order = Order::new(original.plate.clone());
    new_order.original_order_id = Some(order_id);
    new_order.is_courtesy_reorder = true;
    new_order.reordered_by = Some(service.id);
    new_order.reorder_reason = request.reason;

    // Handle priority placement
    if request.priority == Some("high".to_string()) {
        let pending = state.queue.list_pending().await?;
        if let Some(first) = pending.first() {
            new_order.position = first.position - 1.0;
        }
    }

    let created = state.queue.enqueue(new_order).await?;

    Ok((StatusCode::CREATED, Json(created)))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReorderRequest {
    /// Reason for the re-order (required for audit)
    pub reason: Option<String>,
    /// Priority: "high" to place at front of queue
    pub priority: Option<String>,
}
```

### Service Account Configuration

Service accounts are configured via environment variables:

| Variable | Description |
|----------|-------------|
| `SERVICE_ACCOUNT_AI_QA_TOKEN` | Token for AI QA system |
| `SERVICE_ACCOUNT_AI_QA_PERMISSIONS` | Comma-separated permissions |
| `SERVICE_ACCOUNT_PRINTER_TOKEN` | Token for printer system |
| `SERVICE_ACCOUNT_PRINTER_PERMISSIONS` | Comma-separated permissions |

Example configuration:
```bash
SERVICE_ACCOUNT_AI_QA_TOKEN=sk_service_abc123456789
SERVICE_ACCOUNT_AI_QA_PERMISSIONS=reorder_create,order_read_all

SERVICE_ACCOUNT_PRINTER_TOKEN=sk_service_def987654321
SERVICE_ACCOUNT_PRINTER_PERMISSIONS=order_claim,order_complete
```

### Airplane Mode Authentication

In airplane mode, service account tokens are still validated but can be configured locally:

```bash
# .env.local (gitignored)
SERVICE_ACCOUNT_AI_QA_TOKEN=dev_token_ai_qa
SERVICE_ACCOUNT_AI_QA_PERMISSIONS=reorder_create,order_read_all
```

Or bypass authentication entirely for local development:

```bash
SKIP_SERVICE_AUTH=true just dev
```

When `SKIP_SERVICE_AUTH=true`:
- All service endpoints accept any Bearer token
- Service account defaults to `dev-service` with all permissions
- **Never enable in production**

## Implementation

### Module Structure

```
crates/web/src/
├── lib.rs              # Add order routes
├── queue.rs            # OrderQueue trait + types (new)
├── queue_aws.rs        # DynamoDB implementation (new)
├── queue_local.rs      # Filesystem implementation (new)
├── queue_memory.rs     # In-memory implementation (new)
└── handlers/
    └── orders.rs       # Order endpoint handlers (new)
```

### AppState Changes

```rust
pub struct AppStateInner {
    pub sessions: RwLock<HashMap<String, SessionData>>,
    pub cache: Arc<dyn ModelCache>,
    pub queue: Arc<dyn OrderQueue>,  // NEW
}
```

### Backend Implementations

#### 1. AwsOrderQueue (Production)

Uses DynamoDB with optimistic locking for claim operations:

```rust
impl OrderQueue for AwsOrderQueue {
    async fn claim_next(&self) -> Result<Option<Order>, QueueError> {
        // Query first pending order by position
        let pending = self.client
            .query()
            .table_name(&self.table)
            .index_name("status-position-index")
            .key_condition_expression("status = :s")
            .expression_attribute_values(":s", AttributeValue::S("pending".into()))
            .limit(1)
            .send()
            .await?;

        let Some(item) = pending.items().first() else {
            return Ok(None);
        };

        // Atomic claim with condition
        let order_id = item.get("order_id")...;
        self.client
            .update_item()
            .table_name(&self.table)
            .key("order_id", AttributeValue::S(order_id.clone()))
            .condition_expression("status = :pending")
            .update_expression("SET status = :processing, claimed_at = :now, claimed_by = :worker")
            .expression_attribute_values(":pending", AttributeValue::S("pending".into()))
            .expression_attribute_values(":processing", AttributeValue::S("processing".into()))
            // ...
            .send()
            .await?;

        // Return claimed order
        self.get(&order_id).await
    }
}
```

#### 2. LocalOrderQueue (Airplane Mode)

File-based queue for local development:

```
./queue/
├── pending/
│   ├── 001.000000_550e8400.json
│   └── 002.000000_660e8400.json
├── processing/
├── completed/
└── failed/
```

Filename format: `{position:010.6}_{order_id}.json`

Sorting by filename gives position ordering. Moving between directories changes status.

```rust
impl OrderQueue for LocalOrderQueue {
    async fn claim_next(&self) -> Result<Option<Order>, QueueError> {
        let pending_dir = self.base_path.join("pending");

        // Get first file (lowest position due to filename sorting)
        let mut entries: Vec<_> = fs::read_dir(&pending_dir)?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        let Some(entry) = entries.first() else {
            return Ok(None);
        };

        // Atomic move to processing directory
        let src = entry.path();
        let dest = self.base_path.join("processing").join(entry.file_name());

        // Use rename for atomicity on same filesystem
        fs::rename(&src, &dest)?;

        // Update order and return
        let mut order: Order = serde_json::from_str(&fs::read_to_string(&dest)?)?;
        order.status = OrderStatus::Processing;
        order.claimed_at = Some(Utc::now());
        fs::write(&dest, serde_json::to_string_pretty(&order)?)?;

        Ok(Some(order))
    }
}
```

#### 3. MemoryOrderQueue (Testing)

In-memory implementation for unit tests:

```rust
pub struct MemoryOrderQueue {
    orders: RwLock<HashMap<String, Order>>,
}
```

### Initialization

```rust
fn create_queue() -> Arc<dyn OrderQueue> {
    if let Ok(table) = std::env::var("DYNAMODB_ORDERS_TABLE") {
        // Production: AWS DynamoDB
        let config = aws_config::load_from_env().await;
        Arc::new(AwsOrderQueue::new(&config, table))
    } else {
        // Development: Local filesystem
        Arc::new(LocalOrderQueue::new("./queue"))
    }
}
```

## Printer Integration

The printer system is an external process that:

1. **Polls for work**: Calls `POST /api/orders/claim` periodically
2. **Processes order**: Generates physical output
3. **Reports result**: Calls `POST /api/orders/{id}/complete` or `/fail`

### Printer Client Example (Rust)

```rust
async fn printer_loop(api_base: &str, worker_id: &str) {
    loop {
        // Try to claim work
        let resp = client
            .post(format!("{api_base}/api/orders/claim"))
            .json(&json!({ "worker_id": worker_id }))
            .send()
            .await?;

        if resp.status() == StatusCode::NO_CONTENT {
            // No work available, wait and retry
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        let order: Order = resp.json().await?;

        // Process the order
        match process_order(&order).await {
            Ok(result) => {
                client
                    .post(format!("{api_base}/api/orders/{}/complete", order.order_id))
                    .json(&json!({
                        "worker_id": worker_id,
                        "result": result
                    }))
                    .send()
                    .await?;
            }
            Err(e) => {
                client
                    .post(format!("{api_base}/api/orders/{}/fail", order.order_id))
                    .json(&json!({
                        "worker_id": worker_id,
                        "error": e.to_string()
                    }))
                    .send()
                    .await?;
            }
        }
    }
}
```

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DYNAMODB_ORDERS_TABLE` | No | - | DynamoDB table for orders (enables AWS mode) |
| `ORDER_TTL_DAYS` | No | 30 | Days to retain completed orders |
| `QUEUE_BASE_PATH` | No | `./queue` | Local queue directory (airplane mode) |
| `SERVICE_ACCOUNT_AI_QA_TOKEN` | No | - | Auth token for AI QA service account |
| `SERVICE_ACCOUNT_AI_QA_PERMISSIONS` | No | - | Permissions for AI QA service |
| `SERVICE_ACCOUNT_PRINTER_TOKEN` | No | - | Auth token for printer service account |
| `SERVICE_ACCOUNT_PRINTER_PERMISSIONS` | No | - | Permissions for printer service |
| `SKIP_SERVICE_AUTH` | No | false | Bypass service auth (dev only, never in prod) |

## Airplane Mode

When `DYNAMODB_ORDERS_TABLE` is not set, the system runs in airplane mode:

- Queue stored in local filesystem (`./queue/`)
- No AWS credentials required
- Full functionality for development and testing
- Data persists across server restarts

```bash
# Run in airplane mode (default)
just dev

# Run with AWS queue
DYNAMODB_ORDERS_TABLE=platerator-orders just dev
```

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_enqueue_and_claim() {
    let queue = MemoryOrderQueue::new();

    let order = Order::new(test_plate());
    queue.enqueue(order.clone()).await.unwrap();

    let claimed = queue.claim_next().await.unwrap().unwrap();
    assert_eq!(claimed.order_id, order.order_id);
    assert_eq!(claimed.status, OrderStatus::Processing);

    // Queue should now be empty
    assert!(queue.claim_next().await.unwrap().is_none());
}

#[tokio::test]
async fn test_reorder() {
    let queue = MemoryOrderQueue::new();

    let a = queue.enqueue(Order::new(test_plate())).await.unwrap();
    let b = queue.enqueue(Order::new(test_plate())).await.unwrap();
    let c = queue.enqueue(Order::new(test_plate())).await.unwrap();

    // Move C to front
    queue.update_position(&c.order_id, 0.5).await.unwrap();

    let pending = queue.list_pending().await.unwrap();
    assert_eq!(pending[0].order_id, c.order_id);
    assert_eq!(pending[1].order_id, a.order_id);
    assert_eq!(pending[2].order_id, b.order_id);
}
```

### Integration Tests

```rust
#[tokio::test]
#[ignore] // Requires local queue setup
async fn test_local_queue_persistence() {
    let temp = tempdir().unwrap();
    let queue = LocalOrderQueue::new(temp.path());

    let order = queue.enqueue(Order::new(test_plate())).await.unwrap();

    // Simulate restart
    drop(queue);
    let queue = LocalOrderQueue::new(temp.path());

    let pending = queue.list_pending().await.unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].order_id, order.order_id);
}
```

## Migration & Deployment

### DynamoDB Table Creation

```bash
aws dynamodb create-table \
  --table-name platerator-orders \
  --attribute-definitions \
    AttributeName=order_id,AttributeType=S \
    AttributeName=status,AttributeType=S \
    AttributeName=position,AttributeType=N \
  --key-schema \
    AttributeName=order_id,KeyType=HASH \
  --global-secondary-indexes \
    'IndexName=status-position-index,KeySchema=[{AttributeName=status,KeyType=HASH},{AttributeName=position,KeyType=RANGE}],Projection={ProjectionType=ALL}' \
  --billing-mode PAY_PER_REQUEST
```

### Terraform Addition

```hcl
resource "aws_dynamodb_table" "orders" {
  name         = "platerator-orders"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "order_id"

  attribute {
    name = "order_id"
    type = "S"
  }

  attribute {
    name = "status"
    type = "S"
  }

  attribute {
    name = "position"
    type = "N"
  }

  global_secondary_index {
    name            = "status-position-index"
    hash_key        = "status"
    range_key       = "position"
    projection_type = "ALL"
  }

  ttl {
    attribute_name = "ttl"
    enabled        = true
  }
}
```

## Security Considerations

1. **Service Authentication**: The claim/complete/fail/reorder endpoints require service account tokens (Bearer auth)
2. **Permission Model**: Each service account has specific permissions (ReorderCreate, OrderClaim, etc.)
3. **Authorization**: Users should only see/cancel their own orders (requires user identity)
4. **Rate limiting**: Protect claim endpoint from aggressive polling
5. **Validation**: Validate service account ID matches on complete/fail to prevent hijacking
6. **Audit Trail**: All re-orders record who initiated them and why
7. **Token Security**: Service tokens should be rotated regularly and stored in secrets manager (not env vars) for production

## Future Enhancements

1. **WebSocket notifications**: Push queue updates to frontend
2. **Batch operations**: Claim multiple orders at once
3. **Priority levels**: Urgent/normal/low priority queues
4. **Retry logic**: Automatic retry of failed orders
5. **Metrics**: Queue depth, processing time, success rate
6. **Dead letter queue**: Capture repeatedly failing orders

## Summary

This architecture provides:

- **Reliable ordering**: DynamoDB GSI ensures consistent queue order
- **Atomic claims**: Conditional updates prevent double-processing
- **Flexible reordering**: Fractional positioning enables efficient priority changes
- **Courtesy re-orders**: AI systems can re-order failed prints at no customer cost
- **Service authentication**: Secure token-based auth for machine-to-machine operations
- **Audit trail**: Full tracking of who initiated re-orders and why
- **Airplane mode**: Full local development without AWS
- **Familiar patterns**: Follows existing `ModelCache` trait pattern
- **Clean separation**: Queue abstraction allows easy testing and backend swapping
