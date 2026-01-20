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
use uuid::Uuid;

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

    Router::new()
        .route("/api/health", get(health))
        .route("/api/validate", post(validate_plate))
        .route("/api/generate", post(generate_plate_model))
        .route("/api/download/step/{session_id}", get(download_step))
        .route("/api/download/gltf/{session_id}", get(download_gltf))
        .fallback_service(serve_dir)
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    let res = OkResponse { ok: true };
    (StatusCode::OK, Json(res)).into_response()
}

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

#[derive(Serialize)]
struct OkResponse {
    ok: bool,
}

#[derive(Serialize)]
struct GenerateSuccessResponse {
    success: bool,
    message: String,
    download_url: String,
    gltf_url: String,
    session_id: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    got_it: bool,
    errors: Vec<String>,
}

#[derive(Serialize)]
struct ValidationSuccessResponse {
    valid: bool,
    message: String,
}

#[derive(Serialize)]
struct ValidationErrorResponse {
    valid: bool,
    errors: Vec<String>,
}
