use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::extract::{ConnectInfo, State};
use axum::Json;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::response_token::ResponseClaims;
use crate::{db, response_token, AppState};

use super::errors::ChallengeError;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetChallenge {
    pub url: String,
    pub width: u16,
    pub height: u16,
}

#[instrument(skip(state))]
pub async fn get_challenge(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> super::Result<Json<GetChallenge>> {
    let challenge = &state.challenges[0];
    let url = Url::parse(&challenge.url).context("malformed challenge url in config")?;

    Ok(Json(GetChallenge {
        url: url.to_string(),
        width: challenge.width,
        height: challenge.height,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResults {
    // this should be more complex and computed server side
    pub success: bool,
    pub secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub token: String,
}

#[instrument(skip(state))]
pub async fn process_challenge(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(results): Json<ChallengeResults>,
) -> super::Result<Json<ChallengeResponse>> {
    Ok(Json(ChallengeResponse {
        token: response_token::encode(
            ResponseClaims {
                success: results.success,
                authority: addr,
            },
            &db::fetch_encoding_key(&state.pool, &results.secret)
                .await
                .context("failed to fecth encoding key by api secret while processing challenge")?
                .ok_or(ChallengeError::InvalidSecret)?,
        )?,
    }))
}
