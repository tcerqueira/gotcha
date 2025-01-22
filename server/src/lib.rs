use std::sync::{Arc, LazyLock};

use axum::{Extension, Router};
use configuration::{server_dir, ApplicationConfig};
use extractors::ThisOrigin;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "aws-lambda")]
mod aws_lambda {
    pub use tower::util::MapRequestLayer;
}
#[cfg(feature = "aws-lambda")]
use aws_lambda::*;

pub use configuration::{get_configuration, Config};

pub mod analysis;
pub mod configuration;
pub mod crypto;
pub mod db;
pub mod extractors;
pub mod response_token;
pub mod routes;
mod serde;
pub mod test_helpers;

pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);
pub static HTTP_CACHE_CLIENT: LazyLock<ClientWithMiddleware> = LazyLock::new(|| {
    ClientBuilder::new(Client::new())
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager { path: "/tmp/gotcha/".into() },
            options: HttpCacheOptions::default(),
        }))
        .build()
});

#[derive(Debug)]
pub struct AppState {
    pub pool: PgPool,
    pub auth_origin: String,
}

pub fn app(config: ApplicationConfig, pool: PgPool) -> Router {
    let serve_dir = server_dir().join(config.serve_dir).canonicalize().unwrap();
    tracing::info!("Serving files from: {:?}", serve_dir);

    let state = AppState { pool, auth_origin: config.auth_origin };
    let origin = format!("http://localhost:{}", config.port);

    let router = Router::new()
        .nest("/api", api(state))
        .fallback_service(ServeDir::new(serve_dir))
        .layer(TraceLayer::new_for_http());

    #[cfg(feature = "aws-lambda")]
    let router = router
        .layer(MapRequestLayer::new(extractors::extract_lambda_source_ip))
        .layer(MapRequestLayer::new(extractors::extract_lambda_origin));

    router.layer(Extension(ThisOrigin(origin)))
}

fn api(state: AppState) -> Router {
    let state = Arc::new(state);
    Router::new()
        .merge(routes::verification(&state))
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
    let mut txn = pool.begin().await?;

    let _ = db::with_console_insert_api_key(
        &mut *txn,
        "demo",
        "demo|user",
        "4BdwFU84HLqceCQbE90+U5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q",
        "dHsFxb7mDHNv+cuI1L9GDW8AhXdWzuq/pwKWceDGq1SG4y2WD7zBwtiY2LHWNg3m",
    )
    .await;

    txn.commit().await
}
