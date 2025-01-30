use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{instrument, Level};
use uuid::Uuid;

use super::errors::ConsoleError;
use crate::{
    crypto::{self, KEY_SIZE},
    db::{self, DbApiKey, DbConsole, DbUpdateApiKey, DbUpdateConsole, RowsAffected},
    extractors::User,
    AppState,
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

#[instrument(skip_all, ret(level = Level::INFO))]
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
    let id = db::insert_console(&state.pool, &request.label, &user_id).await?;
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
    pub site_key: String,
    pub secret: String,
    pub label: Option<String>,
}

#[instrument(skip(state), ret(level = Level::INFO))]
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

#[instrument(skip(state), ret(level = Level::INFO))]
pub async fn gen_api_key(
    State(state): State<Arc<AppState>>,
    Path(console_id): Path<Uuid>,
) -> Result<Json<ApiKeyResponse>, ConsoleError> {
    let (site_key, secret) = loop {
        let site_key = crypto::gen_base64_url_safe_key::<KEY_SIZE>();
        let enc_key = crypto::gen_base64_key::<KEY_SIZE>();
        let secret = crypto::gen_base64_key::<KEY_SIZE>();

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
