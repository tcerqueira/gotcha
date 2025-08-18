//! Abstraction for database queries.

use std::{fmt::Debug, ops::DerefMut};

use anyhow::Context;
use sqlx::{PgExecutor, Postgres, Transaction, prelude::*};
use uuid::Uuid;

use crate::encodings::{Base64, UrlSafe};

use super::Error;

pub type Result<T> = ::core::result::Result<T, Error>;

/// Wrapper type for reporting the number of rows affected by updates and deletes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowsAffected(pub u64);

/// Database representation of an api key.
#[derive(Debug, FromRow)]
pub struct DbApiKey {
    #[sqlx(try_from = "String")]
    pub site_key: Base64<UrlSafe>,
    #[sqlx(try_from = "String")]
    pub encoding_key: Base64,
    #[sqlx(try_from = "String")]
    pub secret: Base64,
    pub label: Option<String>,
}

impl TryFrom<DbApiKeyInternal> for DbApiKey {
    type Error = anyhow::Error;

    fn try_from(value: DbApiKeyInternal) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            site_key: value
                .site_key
                .try_into()
                .context("could not convert site_key from string")?,
            encoding_key: value
                .encoding_key
                .try_into()
                .context("could not convert encoding_key from string")?,
            secret: value
                .secret
                .try_into()
                .context("could not convert secret from string")?,
            label: value.label,
        })
    }
}

/// Internal representation to benefit from sqlx compile time checks on queries
#[derive(Debug)]
struct DbApiKeyInternal {
    pub site_key: String,
    pub encoding_key: String,
    pub secret: String,
    pub label: Option<String>,
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

pub async fn fetch_api_key_by_site_key(
    exec: impl PgExecutor<'_> + Send,
    site_key: &Base64<UrlSafe>,
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
    site_key: &Base64<UrlSafe>,
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

pub(crate) async fn with_console_insert_api_key(
    exec: impl PgExecutor<'_> + Send + Clone,
    console_label: &str,
    user: &str,
    site_key: &Base64<UrlSafe>,
    enc_key: &Base64,
    secret: &Base64,
) -> Result<Uuid> {
    let row = sqlx::query!(
        r#"with
      console as (insert into public.console (label, user_id) values ($1, $2) returning id)
    insert into
      public.api_key (site_key, console_id, encoding_key, secret)
    values
      (
        $3,
        (select id from console),
        $4, $5
      ) returning console_id"#,
        console_label,
        user,
        site_key.as_str(),
        enc_key.as_str(),
        secret.as_str(),
    )
    .fetch_one(exec.clone())
    .await?;

    insert_challenge_customization(exec, &row.console_id, &DbChallengeCustomization::default())
        .await?;

    Ok(row.console_id)
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

#[derive(Debug)]
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
    txn: &mut Transaction<'_, Postgres>,
    label: &str,
    user: &str,
) -> Result<Uuid> {
    let console_id = insert_only_console(txn.deref_mut(), label, user).await?;
    insert_challenge_customization(
        txn.deref_mut(),
        &console_id,
        &DbChallengeCustomization::default(),
    )
    .await?;
    Ok(console_id)
}

async fn insert_only_console(
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

#[derive(Debug)]
pub struct DbChallenge {
    pub url: String,
    pub label: Option<String>,
    pub width: i16,
    pub height: i16,
    pub small_width: i16,
    pub small_height: i16,
    pub logo_url: Option<String>,
}

impl DbChallenge {
    pub fn new(url: String) -> Self {
        Self {
            url,
            label: None,
            width: 360,
            height: 500,
            small_width: 360,
            small_height: 500,
            logo_url: None,
        }
    }
}

pub async fn fetch_challenges(exec: impl PgExecutor<'_> + Send) -> Result<Vec<DbChallenge>> {
    sqlx::query_as!(
        DbChallenge,
        "select
            url,
            label,
            default_width as width,
            default_height as height,
            default_width as small_width,
            default_height as small_height,
            default_logo_url as logo_url
        from challenge"
    )
    .fetch_all(exec)
    .await
    .map(Ok)?
}

pub async fn fetch_challenges_with_customization(
    exec: impl PgExecutor<'_> + Send,
    site_key: &Base64<UrlSafe>,
) -> Result<Vec<DbChallenge>> {
    sqlx::query_as!(
        DbChallenge,
        "select
            c.url,
            c.label,
            coalesce(cc.width, c.default_width) as \"width!\",
            coalesce(cc.height, c.default_height) as \"height!\",
            coalesce(cc.small_width, c.default_width) as \"small_width!\",
            coalesce(cc.small_height, c.default_height) as \"small_height!\",
            coalesce(cc.logo_url, c.default_logo_url) as logo_url
        from public.challenge c
        left join public.challenge_customization cc on cc.console_id = (
            select console_id
            from public.api_key
            where site_key = $1
        )",
        site_key.as_str(),
    )
    .fetch_all(exec)
    .await
    .map(Ok)?
}

pub async fn insert_challenge(
    exec: impl PgExecutor<'_> + Send,
    challenge: &DbChallenge,
) -> Result<()> {
    sqlx::query!(
        "insert into challenge (url, default_width, default_height, default_logo_url) values ($1, $2, $3, $4)",
        challenge.url,
        challenge.width,
        challenge.height,
        challenge.logo_url
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

#[derive(Debug, PartialEq, Eq)]
pub struct DbChallengeCustomization {
    pub width: i16,
    pub height: i16,
    pub small_width: i16,
    pub small_height: i16,
    pub logo_url: Option<String>,
}

impl Default for DbChallengeCustomization {
    fn default() -> Self {
        Self {
            width: 360,
            height: 500,
            small_width: 360,
            small_height: 500,
            logo_url: None,
        }
    }
}

pub async fn fetch_challenge_customization(
    exec: impl PgExecutor<'_> + Send,
    console_id: &Uuid,
) -> Result<Option<DbChallengeCustomization>> {
    sqlx::query_as!(
        DbChallengeCustomization,
        "select width, height, small_width, small_height, logo_url from challenge_customization where console_id = $1",
        console_id
    )
    .fetch_optional(exec)
    .await
    .map(Ok)?
}

pub async fn insert_challenge_customization(
    exec: impl PgExecutor<'_> + Send,
    console_id: &Uuid,
    insert: &DbChallengeCustomization,
) -> Result<()> {
    sqlx::query_as!(
        DbChallengeCustomization,
        "insert into challenge_customization (console_id, width, height, small_width, small_height, logo_url) values ($1, $2, $3, $4, $5, $6)",
        console_id,
        insert.width,
        insert.height,
        insert.small_width,
        insert.small_height,
        insert.logo_url,
    )
    .execute(exec)
    .await?;
    Ok(())
}

#[derive(Debug)]
pub struct DbUpdateChallengeCustomization<'a> {
    pub width: Option<i16>,
    pub height: Option<i16>,
    pub small_width: Option<i16>,
    pub small_height: Option<i16>,
    pub logo_url: Option<Option<&'a str>>,
}

pub async fn update_challenge_customization(
    exec: impl PgExecutor<'_> + Send,
    console_id: &Uuid,
    update: &DbUpdateChallengeCustomization<'_>,
) -> Result<RowsAffected> {
    let (should_update_logo_url, logo_url_value) = match update.logo_url {
        None => (false, None),
        Some(value) => (true, value),
    };

    let res = sqlx::query!(
        "update challenge_customization set
            width = coalesce($1, width),
            height = coalesce($2, height),
            small_width = coalesce($3, small_width),
            small_height = coalesce($4, small_height),
            logo_url = case when $5 then $6 else logo_url end
        where console_id = $7",
        update.width,
        update.height,
        update.small_width,
        update.small_height,
        should_update_logo_url,
        logo_url_value,
        console_id
    )
    .execute(exec)
    .await?;

    match res.rows_affected() {
        0 => Err(Error::NotFound),
        r => Ok(RowsAffected(r)),
    }
}
