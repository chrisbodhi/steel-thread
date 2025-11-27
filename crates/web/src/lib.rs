use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use domain::ActuatorPlate;
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

use validation::validate;

pub async fn run() {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/plate", post(create_plate))
        .route("/", get(|| async { Redirect::permanent("/order/") }))
        .nest_service("/order", ServeDir::new("crates/web/src/static"));
    let addr = SocketAddr::from(([0, 0, 0, 0], 3030));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, router).await.unwrap()
}

async fn create_plate(Json(payload): Json<ActuatorPlate>) -> impl IntoResponse {
    // Axum's Json extractor already validated the JSON structure
    // Now validate the business rules
    match validate(&payload) {
        Ok(_) => {
            let res = Res {
                got_it: payload.bolt_diameter.0 > 0,
            };
            (StatusCode::CREATED, Json(res))
        }
        Err(e) => {
            tracing::error!("validation error: {}", e);
            let res = Res { got_it: false };
            (StatusCode::BAD_REQUEST, Json(res))
        }
    }
}

#[derive(Serialize)]
struct Res {
    got_it: bool,
}
