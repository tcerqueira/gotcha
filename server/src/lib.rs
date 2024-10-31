use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use configuration::current_crate_dir;
pub use configuration::{get_configuration, Config};
use routes::{
    internal::{get_challenge, process_challenge},
    public::site_verify,
};
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod configuration;
pub mod db;
pub mod response_token;
pub mod routes;
pub mod test_helpers;

#[derive(Debug)]
pub struct AppState {
    pub challenges: Vec<Challenge>,
    pub pool: PgPool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Challenge {
    url: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    width: u16,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    height: u16,
}

pub async fn app(config: Config) -> Router {
    let serve_dir = current_crate_dir()
        .join(config.application.serve_dir)
        .canonicalize()
        .unwrap();
    tracing::debug!("Serving files from: {:?}", serve_dir);

    let state = AppState {
        challenges: config.challenges,
        pool: db::connect_database(config.database),
    };

    Router::new()
        .nest("/api", api(state))
        .fallback_service(ServeDir::new(serve_dir))
        .layer(TraceLayer::new_for_http())
}

fn api(state: AppState) -> Router {
    let state = Arc::new(state);
    Router::new()
        .route("/challenge", get(get_challenge))
        .route("/process-challenge", post(process_challenge))
        .route("/siteverify", post(site_verify))
        .layer(CorsLayer::permissive())
        .with_state(state)
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
