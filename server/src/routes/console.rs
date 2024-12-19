use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
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
pub struct ConsoleRequest {
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleResponse {
    pub id: uuid::Uuid,
}

#[instrument(skip(state))]
pub async fn create_console(
    State(state): State<Arc<AppState>>,
    User { user_id, .. }: User,
    Json(request): Json<ConsoleRequest>,
) -> Result<Json<ConsoleResponse>, ConsoleError> {
    let id = db::insert_console(&state.pool, &request.label, &user_id).await?;
    Ok(Json(ConsoleResponse { id }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSecretRequest {
    pub console_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSecretResponse {
    pub secret: String,
}

#[instrument(skip(state))]
pub async fn gen_api_secret(
    State(state): State<Arc<AppState>>,
    User { user_id, .. }: User,
    Json(request): Json<ApiSecretRequest>,
) -> Result<Json<ApiSecretResponse>, ConsoleError> {
    if !db::exists_console_for_user(&state.pool, &request.console_id, &user_id).await? {
        return Err(ConsoleError::Forbidden);
    }

    let secret = crypto::gen_base64_key::<KEY_SIZE>();
    let enc_key = crypto::gen_base64_key::<KEY_SIZE>();

    db::insert_api_secret(&state.pool, &secret, &request.console_id, &enc_key).await?;
    Ok(Json(ApiSecretResponse { secret }))
}

pub async fn add_origin() -> StatusCode {
    todo!()
}

pub async fn remove_origin() -> StatusCode {
    todo!()
}
