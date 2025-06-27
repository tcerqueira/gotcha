use std::sync::Arc;

use anyhow::Context;
use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use tracing::{Level, instrument};
use uuid::Uuid;

use super::{errors::ConsoleError, extractors::User};
use crate::{
    AppState,
    db::{
        self, DbApiKey, DbChallengeCustomization, DbConsole, DbUpdateApiKey,
        DbUpdateChallengeCustomization, DbUpdateConsole, RowsAffected,
    },
    encodings::{Base64, KEY_SIZE, Standard, UrlSafe},
    serde::nested_option,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConsoleRequest {
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleResponse {
    pub id: Uuid,
    pub label: Option<String>,
}

#[instrument(skip_all, ret(Debug, level = Level::INFO), err(Debug, level = Level::ERROR))]
pub async fn get_consoles(
    State(state): State<Arc<AppState>>,
    User { user_id }: User,
) -> Result<Json<Vec<ConsoleResponse>>, ConsoleError> {
    let consoles = db::fetch_consoles(&state.pool, &user_id)
        .await?
        .into_iter()
        .map(ConsoleResponse::from)
        .collect();

    Ok(Json(consoles))
}

#[instrument(skip(state, user_id), ret(level = Level::INFO))]
pub async fn create_console(
    State(state): State<Arc<AppState>>,
    User { user_id }: User,
    Json(request): Json<CreateConsoleRequest>,
) -> Result<Json<ConsoleResponse>, ConsoleError> {
    let mut txn = state
        .pool
        .begin()
        .await
        .context("db could not begin transaction")?;
    let id = db::insert_console(&mut txn, &request.label, &user_id).await?;
    txn.commit()
        .await
        .context("db could not commit transaction")?;
    Ok(Json(ConsoleResponse { id, label: Some(request.label) }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConsoleRequest {
    pub label: Option<String>,
}

#[instrument(skip(state), err(Debug, level = Level::ERROR))]
pub async fn update_console(
    State(state): State<Arc<AppState>>,
    Path(console_id): Path<Uuid>,
    Json(request): Json<UpdateConsoleRequest>,
) -> Result<(), ConsoleError> {
    let update = DbUpdateConsole { label: request.label.as_deref() };
    match db::update_console(&state.pool, &console_id, update).await? {
        RowsAffected(0) => {
            Err(ConsoleError::NotFound { what: format!("console with id {console_id}") })
        }
        RowsAffected(_) => Ok(()),
    }
}

#[instrument(skip(state), err(Debug, level = Level::ERROR))]
pub async fn delete_console(
    State(state): State<Arc<AppState>>,
    Path(console_id): Path<Uuid>,
) -> Result<(), ConsoleError> {
    match db::delete_console(&state.pool, &console_id).await? {
        RowsAffected(0) => {
            Err(ConsoleError::NotFound { what: format!("console with id {console_id}") })
        }
        RowsAffected(_) => Ok(()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub site_key: Base64<UrlSafe>,
    pub secret: Base64,
    pub label: Option<String>,
}

#[instrument(skip(state), ret(level = Level::INFO), err(Debug, level = Level::ERROR))]
pub async fn get_api_keys(
    State(state): State<Arc<AppState>>,
    Path(console_id): Path<Uuid>,
) -> Result<Json<Vec<ApiKeyResponse>>, ConsoleError> {
    let keys = db::fetch_api_keys(&state.pool, &console_id)
        .await
        .with_context(|| format!("failed to fetch api keys for console id '{console_id}'"))?
        .into_iter()
        .map(ApiKeyResponse::from)
        .collect();

    Ok(Json(keys))
}

#[instrument(skip(state), ret(level = Level::INFO), err(Debug, level = Level::ERROR))]
pub async fn gen_api_key(
    State(state): State<Arc<AppState>>,
    Path(console_id): Path<Uuid>,
) -> Result<Json<ApiKeyResponse>, ConsoleError> {
    let (site_key, secret) = loop {
        let site_key = Base64::<UrlSafe>::random::<KEY_SIZE>();
        let enc_key = Base64::<Standard>::random::<KEY_SIZE>();
        let secret = Base64::<Standard>::random::<KEY_SIZE>();

        match db::insert_api_key(&state.pool, &site_key, &console_id, &enc_key, &secret)
            .await
            .map_err(ConsoleError::from)
        {
            Ok(()) => break (site_key, secret),
            Err(ConsoleError::Duplicate) => continue,
            Err(err) => return Err(err),
        };
    };
    Ok(Json(ApiKeyResponse { site_key, secret, label: None }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub label: Option<String>,
}

#[instrument(skip(state), err(Debug, level = Level::ERROR))]
pub async fn update_api_key(
    State(state): State<Arc<AppState>>,
    Path((console_id, site_key)): Path<(Uuid, String)>,
    Json(request): Json<UpdateApiKeyRequest>,
) -> Result<(), ConsoleError> {
    let update = DbUpdateApiKey { label: request.label.as_deref() };
    match db::update_api_key(&state.pool, &site_key, &console_id, update)
        .await
        .with_context(|| {
            format!("failed to update api key '{site_key}' for console id '{console_id}'")
        })? {
        RowsAffected(0) => Err(ConsoleError::NotFound {
            what: format!("sitekey {site_key} for console with id {console_id}"),
        }),
        RowsAffected(_) => Ok(()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevokeKeyRequest {
    pub site_key: String,
}

#[instrument(skip(state), err(Debug, level = Level::ERROR))]
pub async fn revoke_api_key(
    State(state): State<Arc<AppState>>,
    Path((console_id, site_key)): Path<(Uuid, String)>,
) -> Result<(), ConsoleError> {
    match db::delete_api_key(&state.pool, &site_key, &console_id)
        .await
        .with_context(|| {
            format!("failed to delete api key '{site_key}' for console id '{console_id}'")
        })? {
        RowsAffected(0) => Err(ConsoleError::NotFound {
            what: format!("sitekey {site_key} for console with id {console_id}"),
        }),
        RowsAffected(_) => Ok(()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengePreferences {
    pub width: u16,
    pub height: u16,
    pub small_width: u16,
    pub small_height: u16,
    pub logo_url: Option<String>,
}

impl Default for ChallengePreferences {
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

#[instrument(skip(state), err(Debug, level = Level::ERROR))]
pub async fn get_challenge_preferences(
    State(state): State<Arc<AppState>>,
    Path(console_id): Path<Uuid>,
) -> Result<Json<ChallengePreferences>, ConsoleError> {
    let challenge_preferences = db::fetch_challenge_customization(&state.pool, &console_id)
        .await?
        .map(Into::into)
        .unwrap_or_default();
    Ok(Json(challenge_preferences))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateChallengePreferences {
    #[serde(default)]
    pub width: Option<u16>,
    #[serde(default)]
    pub height: Option<u16>,
    #[serde(default)]
    pub small_width: Option<u16>,
    #[serde(default)]
    pub small_height: Option<u16>,
    #[serde(default, deserialize_with = "nested_option::deserialize")]
    pub logo_url: Option<Option<String>>,
}

#[instrument(skip(state), err(Debug, level = Level::ERROR))]
pub async fn update_challenge_preferences(
    State(state): State<Arc<AppState>>,
    Path(console_id): Path<Uuid>,
    Json(update): Json<UpdateChallengePreferences>,
) -> Result<(), ConsoleError> {
    let res = db::update_challenge_customization(
        &state.pool,
        &console_id,
        &DbUpdateChallengeCustomization {
            width: update.width.map(|x| x as i16),
            height: update.height.map(|x| x as i16),
            small_width: update.small_width.map(|x| x as i16),
            small_height: update.small_height.map(|x| x as i16),
            logo_url: update.logo_url.as_ref().map(|l| l.as_deref()),
        },
    )
    .await?;

    match res {
        RowsAffected(0) => Err(ConsoleError::NotFound {
            what: format!("challenge preferences for console with id {console_id}"),
        }),
        RowsAffected(_) => Ok(()),
    }
}

impl From<DbConsole> for ConsoleResponse {
    fn from(c: DbConsole) -> Self {
        ConsoleResponse { id: c.id, label: c.label }
    }
}

impl From<DbApiKey> for ApiKeyResponse {
    fn from(k: DbApiKey) -> Self {
        ApiKeyResponse { site_key: k.site_key, secret: k.secret, label: k.label }
    }
}

impl From<DbChallengeCustomization> for ChallengePreferences {
    fn from(c: DbChallengeCustomization) -> Self {
        ChallengePreferences {
            height: c.height as u16,
            width: c.width as u16,
            small_width: c.small_width as u16,
            small_height: c.small_height as u16,
            logo_url: c.logo_url,
        }
    }
}
