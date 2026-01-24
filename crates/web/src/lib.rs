use axum::{
    extract::{Json, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use domain::ActuatorPlate;
use parametric::{generate_model, GenerationResult};
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        validate_plate,
        generate_plate_model,
        download_step,
        download_gltf,
    ),
    components(
        schemas(
            ActuatorPlate,
            domain::Millimeters,
            domain::BoltSize,
            OkResponse,
            ValidationSuccessResponse,
            ValidationErrorResponse,
            GenerateSuccessResponse,
            ErrorResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "validation", description = "Plate parameter validation endpoints"),
        (name = "generation", description = "Model generation and download endpoints"),
    ),
    info(
        title = "Platerator API",
        version = "1.0.0",
        description = "REST API for generating actuator plate STEP and glTF model files",
        contact(
            name = "Platerator Team"
        )
    )
)]
pub struct ApiDoc;

/// Shared application state for storing generation results
pub type AppState = Arc<RwLock<HashMap<String, GenerationResult>>>;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let state: AppState = Arc::new(RwLock::new(HashMap::new()));
    let app = create_router(state);

    // Read port from environment variable (for App Runner compatibility)
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse()
        .unwrap_or(3030);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

pub fn create_router(state: AppState) -> Router {
    // Serve static files from dist/, fallback to index.html for SPA routing
    let serve_dir = ServeDir::new("dist").fallback(ServeFile::new("dist/index.html"));

    // Create API routes
    let api_routes = Router::new()
        .route("/api/health", get(health))
        .route("/api/validate", post(validate_plate))
        .route("/api/generate", post(generate_plate_model))
        .route("/api/download/step/{session_id}", get(download_step))
        .route("/api/download/gltf/{session_id}", get(download_gltf))
        .with_state(state);

    // Merge with Swagger UI
    api_routes
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
        .fallback_service(serve_dir)
}

/// Health check endpoint
///
/// Returns a simple OK response to verify the API is running.
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = OkResponse)
    )
)]
async fn health() -> impl IntoResponse {
    let res = OkResponse { ok: true };
    (StatusCode::OK, Json(res)).into_response()
}

/// Validate actuator plate parameters
///
/// Validates the actuator plate configuration without generating model files.
/// Useful for client-side validation before submitting a generation request.
#[utoipa::path(
    post,
    path = "/api/validate",
    tag = "validation",
    request_body = ActuatorPlate,
    responses(
        (status = 200, description = "Plate parameters are valid", body = ValidationSuccessResponse),
        (status = 400, description = "Plate parameters are invalid", body = ValidationErrorResponse)
    )
)]
async fn validate_plate(Json(payload): Json<ActuatorPlate>) -> impl IntoResponse {
    match validation::validate(&payload) {
        Ok(()) => {
            let res = ValidationSuccessResponse {
                valid: true,
                message: "Actuator plate parameters are valid".to_string(),
            };
            (StatusCode::OK, Json(res)).into_response()
        }
        Err(e) => {
            let res = ValidationErrorResponse {
                valid: false,
                errors: vec![e.to_string()],
            };
            (StatusCode::BAD_REQUEST, Json(res)).into_response()
        }
    }
}

/// Generate actuator plate model files
///
/// Generates STEP and glTF model files based on the provided actuator plate configuration.
/// Returns download URLs for the generated files along with a session ID for retrieval.
#[utoipa::path(
    post,
    path = "/api/generate",
    tag = "generation",
    request_body = ActuatorPlate,
    responses(
        (status = 200, description = "Model files generated successfully", body = GenerateSuccessResponse),
        (status = 400, description = "Invalid plate configuration", body = ErrorResponse)
    )
)]
pub async fn generate_plate_model(
    State(state): State<AppState>,
    Json(payload): Json<ActuatorPlate>,
) -> impl IntoResponse {
    match generate_model(&payload) {
        Ok(result) => {
            let session_id = Uuid::new_v4().to_string();
            let download_url = format!("/api/download/step/{}", session_id);
            let gltf_url = format!("/api/download/gltf/{}", session_id);

            // Store the generation result
            {
                let mut sessions = state.write().await;
                sessions.insert(session_id.clone(), result);
            }

            let res = GenerateSuccessResponse {
                success: true,
                message: "Model files generated successfully".to_string(),
                download_url,
                gltf_url,
                session_id,
            };
            (StatusCode::OK, Json(res)).into_response()
        }
        Err(e) => {
            tracing::error!("generation error: {:?}", e);
            let error_msg = match e {
                parametric::AllErrors::ValidationError(msg) => msg,
                parametric::AllErrors::GeneratorError(msg) => msg,
            };
            let res = ErrorResponse {
                success: false,
                got_it: false,
                errors: vec![error_msg],
            };
            (StatusCode::BAD_REQUEST, Json(res)).into_response()
        }
    }
}

/// Download STEP file
///
/// Downloads the generated STEP model file for a given session ID.
/// The session ID is obtained from the generate endpoint response.
#[utoipa::path(
    get,
    path = "/api/download/step/{session_id}",
    tag = "generation",
    params(
        ("session_id" = String, Path, description = "Session ID from the generate endpoint")
    ),
    responses(
        (status = 200, description = "STEP file downloaded successfully", content_type = "application/STEP"),
        (status = 404, description = "Session not found or file not available", body = ErrorResponse)
    )
)]
async fn download_step(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let sessions = state.read().await;

    let Some(result) = sessions.get(&session_id) else {
        let res = ErrorResponse {
            success: false,
            got_it: false,
            errors: vec!["Session not found. Please generate the model first.".to_string()],
        };
        return (StatusCode::NOT_FOUND, Json(res)).into_response();
    };

    match tokio::fs::read(&result.step_file).await {
        Ok(contents) => {
            let headers = [
                (header::CONTENT_TYPE, "application/STEP"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"actuator_plate.step\"",
                ),
            ];
            (StatusCode::OK, headers, contents).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to read STEP file: {}", e);
            let res = ErrorResponse {
                success: false,
                got_it: false,
                errors: vec!["STEP file not found. Please generate the model first.".to_string()],
            };
            (StatusCode::NOT_FOUND, Json(res)).into_response()
        }
    }
}

/// Download glTF file
///
/// Downloads the generated glTF model file for a given session ID.
/// The session ID is obtained from the generate endpoint response.
#[utoipa::path(
    get,
    path = "/api/download/gltf/{session_id}",
    tag = "generation",
    params(
        ("session_id" = String, Path, description = "Session ID from the generate endpoint")
    ),
    responses(
        (status = 200, description = "glTF file downloaded successfully", content_type = "model/gltf+json"),
        (status = 404, description = "Session not found or file not available", body = ErrorResponse)
    )
)]
async fn download_gltf(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let sessions = state.read().await;

    let Some(result) = sessions.get(&session_id) else {
        let res = ErrorResponse {
            success: false,
            got_it: false,
            errors: vec!["Session not found. Please generate the model first.".to_string()],
        };
        return (StatusCode::NOT_FOUND, Json(res)).into_response();
    };

    match tokio::fs::read(&result.gltf_file).await {
        Ok(contents) => {
            let headers = [
                (header::CONTENT_TYPE, "model/gltf+json"),
                (
                    header::CONTENT_DISPOSITION,
                    "inline; filename=\"actuator_plate.gltf\"",
                ),
            ];
            (StatusCode::OK, headers, contents).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to read glTF file: {}", e);
            let res = ErrorResponse {
                success: false,
                got_it: false,
                errors: vec!["glTF file not found. Please generate the model first.".to_string()],
            };
            (StatusCode::NOT_FOUND, Json(res)).into_response()
        }
    }
}

/// Health check response
#[derive(Serialize, ToSchema)]
struct OkResponse {
    /// Always true for successful health checks
    ok: bool,
}

/// Successful model generation response
#[derive(Serialize, ToSchema)]
struct GenerateSuccessResponse {
    /// Always true for successful generation
    success: bool,
    /// Human-readable success message
    message: String,
    /// URL to download the STEP file
    download_url: String,
    /// URL to download the glTF file
    gltf_url: String,
    /// Session ID for retrieving the generated files
    session_id: String,
}

/// Error response
#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    /// Always false for error responses
    success: bool,
    /// Indicates if the error was understood
    got_it: bool,
    /// List of error messages
    errors: Vec<String>,
}

/// Successful validation response
#[derive(Serialize, ToSchema)]
struct ValidationSuccessResponse {
    /// Always true for valid plates
    valid: bool,
    /// Human-readable success message
    message: String,
}

/// Validation error response
#[derive(Serialize, ToSchema)]
struct ValidationErrorResponse {
    /// Always false for invalid plates
    valid: bool,
    /// List of validation error messages
    errors: Vec<String>,
}
