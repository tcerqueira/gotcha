use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgDatabaseError;

use super::errors::ConsoleError;
use crate::{
    crypto::{self, KEY_SIZE},
    db, AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleRequest {
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleResponse {
    pub id: uuid::Uuid,
}

pub async fn create_console(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ConsoleRequest>,
) -> super::Result<Json<ConsoleResponse>> {
    let id = db::insert_console(&state.pool, &request.label).await?;
    Ok(Json(ConsoleResponse { id }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSecretRequest {
    // TODO: require some sort of auth, currently anyone can change any console
    pub console_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSecretResponse {
    pub secret: String,
}

pub async fn gen_api_secret(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ApiSecretRequest>,
) -> super::Result<Json<ApiSecretResponse>> {
    let secret = crypto::gen_base64_key::<KEY_SIZE>();
    let enc_key = crypto::gen_base64_key::<KEY_SIZE>();

    if let Err(db_err) =
        db::insert_api_secret(&state.pool, &secret, &request.console_id, &enc_key).await
    {
        return match db_err {
            sqlx::Error::Database(err)
                if err
                    .downcast_ref::<PgDatabaseError>()
                    .constraint()
                    .is_some_and(|c| c == "api_secret_console_id_fkey") =>
            {
                Err(ConsoleError::Forbidden.into())
            }
            err => Err(anyhow::Error::new(err)
                .context("failed to insert api secret in database while generating one")
                .into()),
        };
    }

    Ok(Json(ApiSecretResponse { secret }))
}

pub async fn add_origin() -> StatusCode {
    todo!()
}

pub async fn remove_origin() -> StatusCode {
    todo!()
}
