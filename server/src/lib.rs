use axum::{
    routing::{get, post},
    Router,
};
use routes::{get_challenge, process_challenge, site_verify};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;
pub mod test_helpers;

pub fn app() -> Router {
    let serve_dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("server")
        .join("dist");
    tracing::debug!("Serving files from: {:?}", serve_dir);

    Router::new()
        .nest("/api", api())
        .fallback_service(ServeDir::new(serve_dir))
        .layer(TraceLayer::new_for_http())
}

fn api() -> Router {
    Router::new()
        .route("/challenge", get(get_challenge))
        .route("/process-challenge", post(process_challenge))
        .route("/siteverify", post(site_verify))
}

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
