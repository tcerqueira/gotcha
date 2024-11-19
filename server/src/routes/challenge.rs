use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{ConnectInfo, State},
    Json,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::errors::ChallengeError;
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> super::Result<Json<GetChallenge>> {
    // let challenge = &state.challenges[0];
    let challenges = db::fetch_challenges(&state.pool).await?;
    let challenge = choose_challenge(challenges);

    Ok(Json(challenge.try_into()?))
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

fn choose_challenge(mut challenges: Vec<DbChallenge>) -> DbChallenge {
    match &challenges[..] {
        [] => DbChallenge {
            url: "http://localhost:8080/im-not-a-robot/index.html".into(),
            width: 304,
            height: 78,
        },
        // _ => challenges.swap_remove(rand::thread_rng().gen_range(0..challenges.len())),
        _ => challenges.swap_remove(0),
    }
}

impl TryFrom<DbChallenge> for GetChallenge {
    type Error = anyhow::Error;

    fn try_from(db_challenge: DbChallenge) -> Result<Self, Self::Error> {
        let url = Url::parse(&db_challenge.url).context("malformed challenge url")?;

        Ok(GetChallenge {
            url: url.to_string(),
            width: db_challenge.width as u16,
            height: db_challenge.height as u16,
        })
    }
}
