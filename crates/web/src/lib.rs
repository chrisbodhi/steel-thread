use axum::{
    extract::Json,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use domain::ActuatorPlate;
use parametric::generate_model;
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};
use validation::validate;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app = create_router();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3030));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

pub fn create_router() -> Router {
    // Serve static files from dist/, fallback to index.html for SPA routing
    let serve_dir = ServeDir::new("dist").fallback(ServeFile::new("dist/index.html"));

    Router::new()
        .route("/api/health", get(health))
        .route("/api/plate", post(create_plate))
        .route("/api/generate", post(generate_plate_model))
        .route("/api/download/step", get(download_step))
        .route("/api/download/gltf", get(download_gltf))
        .fallback_service(serve_dir)
}

async fn health() -> impl IntoResponse {
    let res = OkResponse { ok: true };
    (StatusCode::OK, Json(res)).into_response()
}

pub async fn create_plate(Json(payload): Json<ActuatorPlate>) -> impl IntoResponse {
    match validate(&payload) {
        Ok(_) => {
            let res = SuccessResponse {
                success: true,
                got_it: true,
            };
            (StatusCode::CREATED, Json(res)).into_response()
        }
        Err(e) => {
            tracing::error!("validation error: {}", e);
            let res = ErrorResponse {
                success: false,
                got_it: false,
                errors: vec![e.to_string()],
            };
            (StatusCode::BAD_REQUEST, Json(res)).into_response()
        }
    }
}

pub async fn generate_plate_model(Json(payload): Json<ActuatorPlate>) -> impl IntoResponse {
    match generate_model(&payload) {
        Ok(_) => {
            let res = GenerateSuccessResponse {
                success: true,
                message: "Model files generated successfully".to_string(),
                download_url: "/api/download/step".to_string(),
            };
            (StatusCode::OK, Json(res)).into_response()
        }
        Err(e) => {
            tracing::error!("generation error: {:?}", e);
            let error_msg = match e {
                parametric::AllErrors::ValidationError => {
                    "Validation failed. Please check your plate configuration.".to_string()
                }
                parametric::AllErrors::GeneratorError => {
                    "Failed to generate model files. Please ensure zoo CLI is installed and authenticated.".to_string()
                }
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

async fn download_step() -> impl IntoResponse {
    let file_path = "output_dir/output.step";

    match tokio::fs::read(file_path).await {
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

async fn download_gltf() -> impl IntoResponse {
    let file_path = "output_dir/source.gltf";

    match tokio::fs::read(file_path).await {
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
struct SuccessResponse {
    success: bool,
    got_it: bool,
}

#[derive(Serialize)]
struct GenerateSuccessResponse {
    success: bool,
    message: String,
    download_url: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    got_it: bool,
    errors: Vec<String>,
}
