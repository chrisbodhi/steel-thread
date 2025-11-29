use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
};
use domain::{ActuatorPlate, Millimeters};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn create_test_router() -> Router {
    Router::new()
        .route("/api/health", get(|| async { StatusCode::OK }))
        .route("/api/plate", post(web::create_plate))
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_router().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_plate_valid() {
    let app = create_test_router().await;

    let plate = ActuatorPlate {
        bolt_spacing: Millimeters(60),
        bolt_diameter: Millimeters(10),
        bracket_height: Millimeters(40),
        pin_diameter: Millimeters(10),
        plate_thickness: Millimeters(8),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&plate).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["got_it"], true);
}

#[tokio::test]
async fn test_create_plate_invalid_bolt_spacing() {
    let app = create_test_router().await;

    let plate = ActuatorPlate {
        bolt_spacing: Millimeters(0), // Invalid!
        bolt_diameter: Millimeters(10),
        bracket_height: Millimeters(40),
        pin_diameter: Millimeters(10),
        plate_thickness: Millimeters(8),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&plate).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["got_it"], false);
}

#[tokio::test]
async fn test_create_plate_invalid_json() {
    let app = create_test_router().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plate")
                .header("content-type", "application/json")
                .body(Body::from("{invalid json}"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Axum returns 400 for invalid JSON with the Json extractor
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_plate_all_fields_invalid() {
    let app = create_test_router().await;

    let plate = ActuatorPlate {
        bolt_spacing: Millimeters(0),
        bolt_diameter: Millimeters(0),
        bracket_height: Millimeters(0),
        pin_diameter: Millimeters(0),
        plate_thickness: Millimeters(0),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&plate).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
