//! `/api/admin` routes.

use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use tracing::{Level, instrument};
use url::Url;

use crate::{
    AppState,
    db::{self, RowsAffected},
};

use super::errors::AdminError;

/// Expected payload for new challenge route.
#[derive(Debug, Serialize, Deserialize)]
pub struct AddChallenge {
    /// Public URL.
    pub url: String,
    /// Default width.
    pub width: u16,
    /// Default height.
    pub height: u16,
}

/// Adds a new challenge to the database.
#[instrument(skip(state), err(Debug, level = Level::ERROR))]
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
            small_width: width as i16,
            small_height: height as i16,
            logo_url: None,
            label: None,
        },
    )
    .await?;

    Ok(())
}

/// Expected payload for delete challenge route.
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteChallenge {
    /// Public URL.
    pub url: String,
}

/// Deletes a challenge from the database.
#[instrument(skip(state), err(Debug, level = Level::ERROR))]
pub async fn remove_challenge(
    State(state): State<Arc<AppState>>,
    Json(challenge): Json<DeleteChallenge>,
) -> Result<(), AdminError> {
    let RowsAffected(deleted) = db::delete_challenge(&state.pool, &challenge.url).await?;
    match deleted {
        0 => Err(AdminError::NotFound(challenge.url)),
        _ => Ok(()),
    }
}
