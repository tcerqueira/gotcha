use std::sync::Arc;

use axum::{extract::State, Json};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{db, AppState};

use super::errors::AdminError;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddChallenge {
    pub url: String,
    pub width: u16,
    pub height: u16,
}

#[instrument(skip(state))]
pub async fn add_challenge(
    State(state): State<Arc<AppState>>,
    Json(challenge): Json<AddChallenge>,
) -> Result<(), AdminError> {
    let AddChallenge { url, width, height } = challenge;
    let _ = Url::parse(&url).map_err(|_| AdminError::InvalidUrl)?;

    db::insert_challenge(
        &state.pool,
        &db::DbChallenge {
            url,
            width: width as i16,
            height: height as i16,
        },
    )
    .await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteChallenge {
    pub url: String,
}

#[instrument(skip(state))]
pub async fn remove_challenge(
    State(state): State<Arc<AppState>>,
    Json(challenge): Json<DeleteChallenge>,
) -> Result<(), AdminError> {
    let rows_affected = db::delete_challenge(&state.pool, &challenge.url).await?;
    match rows_affected {
        0 => Err(AdminError::NotFound(challenge.url)),
        _ => Ok(()),
    }
}
