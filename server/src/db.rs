use std::{net::IpAddr, time::Duration};

use secrecy::ExposeSecret;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    types::ipnetwork::IpNetwork,
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

pub struct CreateChallenge<'a> {
    pub api_key: &'a str,
    pub encoding_key: &'a str,
    pub ip_addr: IpAddr,
}

pub async fn create_challenge<'a>(
    pool: impl PgExecutor<'_> + Send,
    create_challenge: &CreateChallenge<'a>,
) -> sqlx::Result<uuid::Uuid> {
    let id: uuid::Uuid = sqlx::query_scalar!(
        "insert into active_challenge (api_key, encoding_key, ip_addr) values ($1, $2, $3) returning id",
        create_challenge.api_key,
        create_challenge.encoding_key,
        IpNetwork::from(create_challenge.ip_addr)
    )
    .fetch_one(pool).await?;
    Ok(id)
}
