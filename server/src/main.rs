mod routes;

use axum::{routing::post, Router};
use routes::site_verify;
use std::net::SocketAddr;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    init_tracing();
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app()).await.unwrap();
}

fn app() -> Router {
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
    Router::new().route("/siteverify", post(site_verify))
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
