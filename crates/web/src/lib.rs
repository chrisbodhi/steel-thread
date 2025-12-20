#[cfg(feature = "ssr")]
use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
#[cfg(feature = "ssr")]
use domain::ActuatorPlate;
#[cfg(feature = "ssr")]
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::{generate_route_list, LeptosRoutes};
#[cfg(feature = "ssr")]
use leptos_meta::{HashedStylesheet, MetaTags};
#[cfg(feature = "ssr")]
use serde::Serialize;

#[cfg(feature = "ssr")]
use validation::validate;

mod app;
mod components;

pub use app::App;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

#[cfg(feature = "ssr")]
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Build Leptos configuration
    let conf = get_configuration(None)?;
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    // Generate routes for Leptos
    let routes = generate_route_list(app::App);

    let router = Router::new()
        // API routes
        .route("/api/health", get(|| async { StatusCode::OK }))
        .route("/api/plate", post(create_plate))
        // Leptos SSR routes
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        // Serve static assets and handle errors
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::debug!("listening on {}", listener.local_addr()?);

    axum::serve(listener, router).await?;

    Ok(())
}

#[cfg(feature = "ssr")]
fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone()/>
                <HashedStylesheet id="leptos" options=options/>
                <MetaTags/>
            </head>
            <body>
                <app::App/>
            </body>
        </html>
    }
}

#[cfg(feature = "ssr")]
pub async fn create_plate(Json(payload): Json<ActuatorPlate>) -> impl IntoResponse {
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
            eprintln!("{}!", e);
            let res = Res { got_it: false };
            (StatusCode::BAD_REQUEST, Json(res))
        }
    }
}

#[cfg(feature = "ssr")]
#[derive(Serialize)]
struct Res {
    got_it: bool,
}
