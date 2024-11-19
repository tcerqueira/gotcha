use std::sync::Arc;

use axum::Router;
use configuration::{current_crate_dir, ApplicationConfig, ChallengeConfig};
use secrecy::Secret;
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use configuration::{get_configuration, Config};

pub mod configuration;
pub mod crypto;
pub mod db;
pub mod response_token;
pub mod routes;
pub mod test_helpers;

#[derive(Debug)]
pub struct AppState {
    pub challenges: Vec<ChallengeConfig>,
    pub pool: PgPool,
    pub admin_auth_key: Secret<String>,
}

pub fn app(config: ApplicationConfig, pool: PgPool) -> Router {
    let serve_dir = current_crate_dir()
        .join(config.serve_dir)
        .canonicalize()
        .unwrap();
    tracing::info!("Serving files from: {:?}", serve_dir);

    let state = AppState {
        challenges: config.challenges,
        pool,
        admin_auth_key: config.admin_auth_key,
    };

    Router::new()
        .nest("/api", api(state))
        .fallback_service(ServeDir::new(serve_dir))
        .layer(TraceLayer::new_for_http())
}

fn api(state: AppState) -> Router {
    let state = Arc::new(state);
    Router::new()
        .nest("/", routes::public(&state))
        .nest("/challenge", routes::challenge(&state))
        .nest("/console", routes::console(&state))
        .nest("/admin", routes::admin(&state))
        .layer(CorsLayer::permissive())
}

pub fn init_tracing() {
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer().with_test_writer())
        .try_init();
}

pub async fn db_dev_populate(pool: &PgPool) -> sqlx::Result<()> {
    db::insert_challenge(
        pool,
        &db::DbChallenge {
            url: "http://localhost:8080/im-not-a-robot/index.html".into(),
            width: 304,
            height: 78,
        },
    )
    .await
}
