use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::extract::ConnectInfo;
use axum::{extract::State, Json};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::errors::ChallengeError;
use crate::analysis::interaction::Event;
use crate::extractors::ThisOrigin;
use crate::{db, response_token, AppState};
use crate::{db::DbChallenge, response_token::ResponseClaims};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetChallenge {
    pub url: String,
    pub width: u16,
    pub height: u16,
}

#[instrument(skip(state))]
pub async fn get_challenge(
    State(state): State<Arc<AppState>>,
    ThisOrigin(origin): ThisOrigin,
) -> Result<Json<GetChallenge>, ChallengeError> {
    let challenges = db::fetch_challenges(&state.pool)
        .await
        .context("failed to fetch challenges")?;
    let challenge = choose_challenge(challenges).unwrap_or_else(|| DbChallenge {
        url: format!("{origin}/im-not-a-robot/index.html"),
        width: 304,
        height: 78,
    });

    Ok(Json(challenge.try_into()?))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResults {
    pub success: bool,
    pub secret: String,
    pub challenge: Option<String>,
    pub interactions: Option<Vec<Event>>,
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
) -> Result<Json<ChallengeResponse>, ChallengeError> {
    Ok(Json(ChallengeResponse {
        token: response_token::encode(
            ResponseClaims { success: results.success, authority: addr },
            &db::fetch_api_key_by_site_key(&state.pool, &results.secret)
                .await
                .context("failed to fecth encoding key by api secret while processing challenge")?
                .ok_or(ChallengeError::InvalidSecret)?
                .encoding_key,
        )?,
    }))
}

fn choose_challenge(mut challenges: Vec<DbChallenge>) -> Option<DbChallenge> {
    match &challenges[..] {
        [] => None,
        // _ => challenges.swap_remove(rand::thread_rng().gen_range(0..challenges.len())),
        _ => Some(challenges.swap_remove(0)),
    }
}

impl TryFrom<DbChallenge> for GetChallenge {
    type Error = anyhow::Error;

    fn try_from(db_challenge: DbChallenge) -> Result<Self, Self::Error> {
        let url = Url::parse(&db_challenge.url)
            .with_context(|| format!("malformed challenge url: {}", db_challenge.url))?;

        Ok(GetChallenge {
            url: url.to_string(),
            width: db_challenge.width as u16,
            height: db_challenge.height as u16,
        })
    }
}
