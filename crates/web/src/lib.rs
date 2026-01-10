use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use domain::ActuatorPlate;
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
struct ErrorResponse {
    success: bool,
    got_it: bool,
    errors: Vec<String>,
}
