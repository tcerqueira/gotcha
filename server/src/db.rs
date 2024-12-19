use std::time::Duration;

use secrecy::ExposeSecret;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    prelude::FromRow,
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

pub async fn fetch_api_secrets(
    exec: impl PgExecutor<'_> + Send,
    console_id: &uuid::Uuid,
) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar!(
        "select key from api_secret where console_id = $1",
        console_id
    )
    .fetch_all(exec)
    .await
}

pub async fn insert_api_secret(
    exec: impl PgExecutor<'_> + Send,
    secret_key: &str,
    console_id: &uuid::Uuid,
    enc_key: &str,
) -> sqlx::Result<()> {
    let _ = sqlx::query!(
        "insert into api_secret (key, console_id, encoding_key, secret) values ($1, $2, $3, $1)",
        secret_key,
        console_id,
        enc_key,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn with_console_insert_api_secret(
    exec: impl PgExecutor<'_> + Send,
    console_label: &str,
    user: &str,
    secret_key: &str,
    enc_key: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"with
      console as (insert into public.console (label, user_id) values ($1, $2) returning id)
    insert into
      public.api_secret (key, console_id, encoding_key, secret)
    values
      (
        $3,
        (select id from console),
        $4, $3
      )"#,
        console_label,
        user,
        secret_key,
        enc_key
    )
    .execute(exec)
    .await?;
    Ok(())
}

#[derive(Debug, FromRow)]
pub struct FetchConsole {
    pub id: uuid::Uuid,
    pub label: Option<String>,
}

pub async fn fetch_consoles(
    exec: impl PgExecutor<'_> + Send,
    user: &str,
) -> sqlx::Result<Vec<FetchConsole>> {
    sqlx::query_as!(
        FetchConsole,
        "select id, label from console where user_id = $1",
        user
    )
    .fetch_all(exec)
    .await
}

pub async fn fetch_console_by_label(
    exec: impl PgExecutor<'_> + Send,
    label: &str,
) -> sqlx::Result<Option<uuid::Uuid>> {
    sqlx::query_scalar!("select id from console where label = $1", label)
        .fetch_optional(exec)
        .await
}

pub async fn exists_console_for_user(
    exec: impl PgExecutor<'_> + Send,
    console_id: &uuid::Uuid,
    user_id: &str,
) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        "select exists (select 1 from console where id = $1 and user_id = $2) as found_console_for_user",
        console_id,
        user_id
    )
    .fetch_one(exec)
    .await
    .map(|r| r.unwrap_or(false))
}

pub async fn insert_console(
    exec: impl PgExecutor<'_> + Send,
    label: &str,
    user: &str,
) -> sqlx::Result<uuid::Uuid> {
    sqlx::query_scalar!(
        "insert into console (label, user_id) values ($1, $2) returning id",
        label,
        user
    )
    .fetch_one(exec)
    .await
}

pub async fn delete_console(
    exec: impl PgExecutor<'_> + Send,
    console_id: &uuid::Uuid,
) -> sqlx::Result<u64> {
    let res = sqlx::query!("delete from console where id = $1", console_id)
        .execute(exec)
        .await?;
    Ok(res.rows_affected())
}

#[derive(Debug, FromRow)]
pub struct DbChallenge {
    pub url: String,
    pub width: i16,
    pub height: i16,
}

pub async fn fetch_challenges(exec: impl PgExecutor<'_> + Send) -> sqlx::Result<Vec<DbChallenge>> {
    sqlx::query_as!(DbChallenge, "select url, width, height from challenge")
        .fetch_all(exec)
        .await
}

pub async fn insert_challenge(
    exec: impl PgExecutor<'_> + Send,
    challenge: &DbChallenge,
) -> sqlx::Result<()> {
    sqlx::query!(
        "insert into challenge (url, width, height) values ($1, $2, $3)",
        challenge.url,
        challenge.width,
        challenge.height
    )
    .execute(exec)
    .await?;
    Ok(())
}

pub async fn delete_challenge(
    exec: impl PgExecutor<'_> + Send,
    challenge_url: &str,
) -> sqlx::Result<u64> {
    let res = sqlx::query!("delete from challenge where url = $1", challenge_url)
        .execute(exec)
        .await?;
    Ok(res.rows_affected())
}

pub async fn delete_challenge_like(
    exec: impl PgExecutor<'_> + Send,
    url_pattern: &str,
) -> sqlx::Result<u64> {
    let res = sqlx::query!("delete from challenge where url like $1", url_pattern)
        .execute(exec)
        .await?;
    Ok(res.rows_affected())
}
