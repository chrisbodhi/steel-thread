use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use domain::{ActuatorPlate, Millimeters};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn create_test_router() -> axum::Router {
    web::create_router()
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_router();

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
async fn test_generate_endpoint_invalid_plate() {
    let app = create_test_router();

    let plate = ActuatorPlate {
        bolt_spacing: Millimeters(0), // Invalid!
        bolt_diameter: Millimeters(10),
        bracket_height: Millimeters(40),
        bracket_width: Millimeters(30),
        pin_diameter: Millimeters(10),
        pin_count: 6,
        plate_thickness: Millimeters(8),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/generate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&plate).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], false);
    assert!(json["errors"].as_array().unwrap().len() > 0);
}

// This test validates that the endpoint is properly wired, but will fail
// to generate files if zoo CLI is not installed. We test that it returns
// the expected error in that case.
#[tokio::test]
async fn test_generate_endpoint_valid_plate() {
    let app = create_test_router();

    let plate = ActuatorPlate {
        bolt_spacing: Millimeters(60),
        bolt_diameter: Millimeters(10),
        bracket_height: Millimeters(40),
        bracket_width: Millimeters(30),
        pin_diameter: Millimeters(10),
        pin_count: 6,
        plate_thickness: Millimeters(8),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/generate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&plate).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Response will be OK if zoo is installed, BAD_REQUEST if not
    // We just verify that the endpoint responds correctly
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Should always have a success field
    assert!(json.get("success").is_some());

    if status == StatusCode::OK {
        // If zoo is installed and authenticated
        assert_eq!(json["success"], true);
        assert!(json.get("message").is_some());
    } else {
        // If zoo is not available
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["success"], false);
        assert!(json["errors"].as_array().unwrap().len() > 0);
    }

    // Cleanup if files were created
    std::fs::remove_file("params.kcl").ok();
    std::fs::remove_file("output_dir/output.step").ok();
    std::fs::remove_file("output_dir/output.gltf").ok();
}
