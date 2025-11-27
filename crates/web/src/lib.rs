use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

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
    // Now we just need to use the validated data
    let res = Res {
        got_it: payload.bolt_diameter.0 > 0,
    };
    (StatusCode::CREATED, Json(res))
}

#[derive(Serialize)]
struct Res {
    got_it: bool,
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, PartialOrd)]
pub struct Millimeters(pub u16);

#[derive(Debug)]
pub enum PlateValidationError {
    BoltSpacingTooSmall,
    BoltDiameterInvalid,
    BracketHeightInvalid,
    PinDiameterInvalid,
    PlateThicknessInvalid,
}

impl std::fmt::Display for PlateValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BoltSpacingTooSmall => write!(f, "bolt spacing must be greater than 0"),
            Self::BoltDiameterInvalid => write!(f, "bolt diameter must be greater than 0"),
            Self::BracketHeightInvalid => write!(f, "bracket height must be greater than 0"),
            Self::PinDiameterInvalid => write!(f, "pin diameter must be greater than 0"),
            Self::PlateThicknessInvalid => write!(f, "plate thickness must be greater than 0"),
        }
    }
}

impl std::error::Error for PlateValidationError {}

#[derive(Deserialize)]
struct ActuatorPlate {
    bolt_spacing: Millimeters,
    bolt_diameter: Millimeters,
    bracket_height: Millimeters,
    pin_diameter: Millimeters,
    plate_thickness: Millimeters,
}

impl ActuatorPlate {
    pub fn new(
        bolt_spacing: Millimeters,
        bolt_diameter: Millimeters,
        bracket_height: Millimeters,
        pin_diameter: Millimeters,
        plate_thickness: Millimeters,
    ) -> Result<Self, PlateValidationError> {
        // Validate each field
        if bolt_spacing.0 == 0 {
            return Err(PlateValidationError::BoltSpacingTooSmall);
        }
        if bolt_diameter.0 == 0 {
            return Err(PlateValidationError::BoltDiameterInvalid);
        }
        if bracket_height.0 == 0 {
            return Err(PlateValidationError::BracketHeightInvalid);
        }
        if pin_diameter.0 == 0 {
            return Err(PlateValidationError::PinDiameterInvalid);
        }
        if plate_thickness.0 == 0 {
            return Err(PlateValidationError::PlateThicknessInvalid);
        }

        Ok(ActuatorPlate {
            bolt_spacing,
            bolt_diameter,
            bracket_height,
            pin_diameter,
            plate_thickness,
        })
    }

    pub fn default() -> Self {
        ActuatorPlate {
            bolt_spacing: Millimeters(60),
            bolt_diameter: Millimeters(10),
            bracket_height: Millimeters(40),
            pin_diameter: Millimeters(10),
            plate_thickness: Millimeters(8),
        }
    }
}
