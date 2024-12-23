use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::errors::ConsoleError;
use crate::{
    crypto::{self, KEY_SIZE},
    db,
    extractors::User,
    AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConsoleRequest {
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleResponse {
    pub id: uuid::Uuid,
    pub label: Option<String>,
}

#[instrument(skip(state))]
pub async fn create_console(
    State(state): State<Arc<AppState>>,
    User { user_id }: User,
    Json(request): Json<CreateConsoleRequest>,
) -> Result<Json<ConsoleResponse>, ConsoleError> {
    let id = db::insert_console(&state.pool, &request.label, &user_id).await?;
    Ok(Json(ConsoleResponse {
        id,
        label: Some(request.label),
    }))
}

#[instrument(skip(state))]
pub async fn get_consoles(
    State(state): State<Arc<AppState>>,
    User { user_id }: User,
) -> Result<Json<Vec<ConsoleResponse>>, ConsoleError> {
    let db_consoles = db::fetch_consoles(&state.pool, &user_id).await?;
    Ok(Json(
        db_consoles
            .into_iter()
            .map(|c| ConsoleResponse {
                id: c.id,
                label: c.label,
            })
            .collect(),
    ))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSecret {
    pub site_key: String,
    pub secret: String,
}

#[instrument(skip(state))]
pub async fn gen_api_secret(
    State(state): State<Arc<AppState>>,
    User { user_id }: User,
    Path(console_id): Path<uuid::Uuid>,
) -> Result<Json<ApiSecret>, ConsoleError> {
    let (site_key, secret) = loop {
        let site_key = crypto::gen_base64_key::<KEY_SIZE>();
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
    Ok(Json(ApiSecret { site_key, secret }))
}

#[instrument(skip(state))]
pub async fn revoke_api_secret(
    State(state): State<Arc<AppState>>,
    User { user_id, .. }: User,
    Path((console_id, site_key)): Path<(uuid::Uuid, String)>,
) -> Result<(), ConsoleError> {
    db::delete_api_key(&state.pool, &site_key, &console_id).await?;
    Ok(())
}
