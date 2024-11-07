use std::time::Duration;

use secrecy::ExposeSecret;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgExecutor, PgPool,
};

use crate::configuration::DatabaseConfig;

pub fn connect_database(config: DatabaseConfig) -> PgPool {
    let pool_options = PgPoolOptions::default().acquire_timeout(Duration::from_secs(5));
    let conn_options = PgConnectOptions::new()
        .host(&config.host)
        .username(&config.username)
        .password(config.password.expose_secret())
        .port(config.port)
        .database(&config.database_name)
        .ssl_mode(match config.require_ssl {
            true => PgSslMode::Require,
            false => PgSslMode::Prefer,
        });

    pool_options.connect_lazy_with(conn_options)
}

pub async fn fetch_encoding_key(
    exec: impl PgExecutor<'_> + Send,
    api_secret: &str,
) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar!(
        "select encoding_key from api_secret where key = $1",
        api_secret
    )
    .fetch_optional(exec)
    .await
}
