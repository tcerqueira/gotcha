use std::time::Duration;

use secrecy::ExposeSecret;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};

use crate::configuration::DatabaseConfig;

pub mod errors;
pub mod queries;

pub use errors::*;
pub use queries::*;

pub fn connect_database(config: DatabaseConfig) -> PgPool {
    let pool_options = PgPoolOptions::default().acquire_timeout(Duration::from_secs(3));
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

// Extension trait to try to map nested types inside a result type.
trait MapNested<T, E> {
    type Output<U>;

    fn map_nested_with<U, E0, F, ErrMap>(
        self,
        f: F,
        err_map: ErrMap,
    ) -> std::result::Result<Self::Output<U>, E>
    where
        F: Fn(T) -> std::result::Result<U, E0>,
        ErrMap: FnOnce(E0) -> E;

    #[expect(dead_code)]
    fn map_nested<U, E0, F>(self, f: F) -> std::result::Result<Self::Output<U>, E>
    where
        F: Fn(T) -> std::result::Result<U, E0>,
        E: From<E0>,
        Self: Sized,
    {
        self.map_nested_with(f, Into::into)
    }
}

impl<T, E> MapNested<T, E> for std::result::Result<Option<T>, E> {
    type Output<U> = Option<U>;

    fn map_nested_with<U, E0, F, ErrMap>(
        self,
        f: F,
        err_map: ErrMap,
    ) -> std::result::Result<Self::Output<U>, E>
    where
        F: Fn(T) -> std::result::Result<U, E0>,
        ErrMap: FnOnce(E0) -> E,
    {
        self.and_then(|opt_t| opt_t.map(f).transpose().map_err(err_map))
    }
}

impl<T, E> MapNested<T, E> for std::result::Result<Vec<T>, E> {
    type Output<U> = Vec<U>;

    fn map_nested_with<U, E0, F, ErrMap>(
        self,
        f: F,
        err_map: ErrMap,
    ) -> std::result::Result<Self::Output<U>, E>
    where
        F: Fn(T) -> std::result::Result<U, E0>,
        ErrMap: FnOnce(E0) -> E,
    {
        self.and_then(|vec| {
            vec.into_iter()
                .map(f)
                .collect::<std::result::Result<Vec<U>, E0>>()
                .map_err(err_map)
        })
    }
}

fn api_key_decode_err(err: anyhow::Error) -> sqlx::Error {
    sqlx::Error::Decode(err.into_boxed_dyn_error())
}
