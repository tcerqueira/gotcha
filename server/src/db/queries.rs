use std::fmt::Debug;

use base64::DecodeError;
use sqlx::{PgExecutor, prelude::*};
use uuid::Uuid;

use crate::crypto::{Base64, Base64UrlSafe};

use super::Error;

pub type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowsAffected(pub u64);

#[derive(Debug, FromRow)]
pub struct DbApiKey {
    #[sqlx(try_from = "String")]
    pub site_key: Base64UrlSafe,
    #[sqlx(try_from = "String")]
    pub encoding_key: Base64,
    #[sqlx(try_from = "String")]
    pub secret: Base64,
    pub label: Option<String>,
}

impl TryFrom<DbApiKeyInternal> for DbApiKey {
    type Error = DecodeError;

    fn try_from(value: DbApiKeyInternal) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            site_key: value.site_key.try_into()?,
            encoding_key: value.encoding_key.try_into()?,
            secret: value.secret.try_into()?,
            label: value.label,
        })
    }
}

// Internal representation to benefit from sqlx compile time checks on queries
#[derive(Debug, FromRow)]
struct DbApiKeyInternal {
    pub site_key: String,
    pub encoding_key: String,
    pub secret: String,
    pub label: Option<String>,
}

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

fn api_key_decode_err(err: DecodeError) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(err))
}

pub async fn fetch_api_key_by_site_key(
    exec: impl PgExecutor<'_> + Send,
    site_key: &Base64UrlSafe,
) -> Result<Option<DbApiKey>> {
    sqlx::query_as!(
        DbApiKeyInternal,
        "select site_key, encoding_key, secret, label from api_key where site_key = $1",
        site_key.as_str()
    )
    .fetch_optional(exec)
    .await
    .map_nested_with(TryFrom::try_from, api_key_decode_err)
    .map(Ok)?
}

pub async fn fetch_api_key_by_secret(
    exec: impl PgExecutor<'_> + Send,
    secret: &Base64,
) -> Result<Option<DbApiKey>> {
    sqlx::query_as!(
        DbApiKeyInternal,
        "select site_key, encoding_key, secret, label from api_key where secret = $1",
        secret.as_str()
    )
    .fetch_optional(exec)
    .await
    .map_nested_with(TryFrom::try_from, api_key_decode_err)
    .map(Ok)?
}

pub async fn fetch_api_keys(
    exec: impl PgExecutor<'_> + Send,
    console_id: &Uuid,
) -> Result<Vec<DbApiKey>> {
    sqlx::query_as!(
        DbApiKeyInternal,
        "select site_key, encoding_key, secret, label from api_key where console_id = $1 order by created_at",
        console_id
    )
    .fetch_all(exec)
    .await
    .map_nested_with(TryFrom::try_from, api_key_decode_err)
    .map(Ok)?
}

pub async fn insert_api_key(
    exec: impl PgExecutor<'_> + Send,
    site_key: &Base64UrlSafe,
    console_id: &Uuid,
    enc_key: &Base64,
    secret: &Base64,
) -> Result<()> {
    let _ = sqlx::query!(
        "insert into api_key (site_key, console_id, encoding_key, secret) values ($1, $2, $3, $4)",
        site_key.as_str(),
        console_id,
        enc_key.as_str(),
        secret.as_str()
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn exists_api_key_for_console(
    exec: impl PgExecutor<'_> + Send,
    site_key: &str,
    console_id: &Uuid,
) -> Result<bool> {
    sqlx::query_scalar!(
        "select exists (select 1 from api_key where site_key = $1 and console_id = $2) as found_site_key_for_console",
        site_key,
        console_id,
    )
    .fetch_one(exec)
    .await
    .map(|r| r.unwrap_or(false))
    .map(Ok)?
}

pub async fn with_console_insert_api_key(
    exec: impl PgExecutor<'_> + Send,
    console_label: &str,
    user: &str,
    site_key: &Base64UrlSafe,
    enc_key: &Base64,
    secret: &Base64,
) -> Result<()> {
    sqlx::query!(
        r#"with
      console as (insert into public.console (label, user_id) values ($1, $2) returning id)
    insert into
      public.api_key (site_key, console_id, encoding_key, secret)
    values
      (
        $3,
        (select id from console),
        $4, $5
      )"#,
        console_label,
        user,
        site_key.as_str(),
        enc_key.as_str(),
        secret.as_str(),
    )
    .execute(exec)
    .await?;
    Ok(())
}

#[derive(Debug)]
pub struct DbUpdateApiKey<'a> {
    pub label: Option<&'a str>,
}

pub async fn update_api_key(
    exec: impl PgExecutor<'_> + Send,
    site_key: &str,
    console_id: &Uuid,
    update: DbUpdateApiKey<'_>,
) -> Result<RowsAffected> {
    let res = sqlx::query!(
        "update api_key set label = coalesce($1, label) where site_key = $2 and console_id = $3",
        update.label,
        site_key,
        console_id
    )
    .execute(exec)
    .await?;

    match res.rows_affected() {
        0 => Err(Error::NotFound),
        r => Ok(RowsAffected(r)),
    }
}

pub async fn delete_api_key(
    exec: impl PgExecutor<'_> + Send,
    site_key: &str,
    console_id: &Uuid,
) -> Result<RowsAffected> {
    let res = sqlx::query!(
        "delete from api_key where site_key = $1 and console_id = $2",
        site_key,
        console_id
    )
    .execute(exec)
    .await?;
    Ok(RowsAffected(res.rows_affected()))
}

#[derive(Debug, FromRow)]
pub struct DbConsole {
    pub id: Uuid,
    pub label: Option<String>,
}

pub async fn fetch_consoles(
    exec: impl PgExecutor<'_> + Send,
    user: &str,
) -> Result<Vec<DbConsole>> {
    sqlx::query_as!(
        DbConsole,
        "select id, label from console where user_id = $1 order by created_at",
        user
    )
    .fetch_all(exec)
    .await
    .map(Ok)?
}

pub async fn fetch_console_by_label(
    exec: impl PgExecutor<'_> + Send,
    label: &str,
) -> Result<Option<Uuid>> {
    sqlx::query_scalar!("select id from console where label = $1", label)
        .fetch_optional(exec)
        .await
        .map(Ok)?
}

pub async fn exists_console_for_user(
    exec: impl PgExecutor<'_> + Send,
    console_id: &Uuid,
    user_id: &str,
) -> Result<bool> {
    sqlx::query_scalar!(
        "select exists (select 1 from console where id = $1 and user_id = $2) as found_console_for_user",
        console_id,
        user_id
    )
    .fetch_one(exec)
    .await
    .map(|r| r.unwrap_or(false))
    .map(Ok)?
}

pub async fn insert_console(
    exec: impl PgExecutor<'_> + Send,
    label: &str,
    user: &str,
) -> Result<Uuid> {
    sqlx::query_scalar!(
        "insert into console (label, user_id) values ($1, $2) returning id",
        label,
        user
    )
    .fetch_one(exec)
    .await
    .map(Ok)?
}

#[derive(Debug)]
pub struct DbUpdateConsole<'a> {
    pub label: Option<&'a str>,
}

pub async fn update_console(
    exec: impl PgExecutor<'_> + Send,
    id: &Uuid,
    update: DbUpdateConsole<'_>,
) -> Result<RowsAffected> {
    let res = sqlx::query!(
        "update console set label = coalesce($1, label) where id = $2",
        update.label,
        id
    )
    .execute(exec)
    .await?;

    match res.rows_affected() {
        0 => Err(Error::NotFound),
        r => Ok(RowsAffected(r)),
    }
}

pub async fn delete_console(
    exec: impl PgExecutor<'_> + Send,
    console_id: &Uuid,
) -> Result<RowsAffected> {
    let res = sqlx::query!("delete from console where id = $1", console_id)
        .execute(exec)
        .await?;
    Ok(RowsAffected(res.rows_affected()))
}

#[derive(Debug, FromRow)]
pub struct DbChallenge {
    pub url: String,
    pub width: i16,
    pub height: i16,
}

pub async fn fetch_challenges(exec: impl PgExecutor<'_> + Send) -> Result<Vec<DbChallenge>> {
    sqlx::query_as!(DbChallenge, "select url, width, height from challenge")
        .fetch_all(exec)
        .await
        .map(Ok)?
}

pub async fn insert_challenge(
    exec: impl PgExecutor<'_> + Send,
    challenge: &DbChallenge,
) -> Result<()> {
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
) -> Result<RowsAffected> {
    let res = sqlx::query!("delete from challenge where url = $1", challenge_url)
        .execute(exec)
        .await?;
    Ok(RowsAffected(res.rows_affected()))
}

pub async fn delete_challenge_like(
    exec: impl PgExecutor<'_> + Send,
    url_pattern: &str,
) -> Result<RowsAffected> {
    let res = sqlx::query!("delete from challenge where url like $1", url_pattern)
        .execute(exec)
        .await?;
    Ok(RowsAffected(res.rows_affected()))
}
