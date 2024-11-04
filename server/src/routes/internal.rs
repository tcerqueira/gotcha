use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{ConnectInfo, Query, State};
use axum::Json;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::instrument;

use crate::db::CreateChallenge;
use crate::{db, response_token, AppState};

#[derive(Debug, Deserialize)]
pub struct QueryChallenge {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetChallenge {
    pub id: uuid::Uuid,
    pub url: String,
    pub width: u16,
    pub height: u16,
}

#[instrument(skip(state))]
pub async fn get_challenge(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryChallenge>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> super::Result<Json<GetChallenge>> {
    let challenge = &state.challenges[0];
    let mut url = Url::parse(&challenge.url).expect("malformed challenge url in config");
    url.query_pairs_mut().append_pair("token", &params.token);

    let id = db::create_challenge(
        &state.pool,
        &CreateChallenge {
            api_key: &params.token,
            encoding_key: "aaa",
            ip_addr: addr.ip(),
        },
    )
    .await?;

    Ok(Json(GetChallenge {
        id,
        url: url.to_string(),
        width: challenge.width,
        height: challenge.height,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResults {
    pub challenge_id: uuid::Uuid,
    // this should be more complex and computed server side
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub token: String,
}

#[instrument]
pub async fn process_challenge(
    Json(results): Json<ChallengeResults>,
) -> super::Result<Json<ChallengeResponse>> {
    Ok(Json(ChallengeResponse {
        token: response_token::encode(ResponseClaims {
            success: results.success,
        })?,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    #[serde(with = "time::serde::timestamp")]
    exp: OffsetDateTime,
    #[serde(flatten)]
    pub custom: ResponseClaims,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseClaims {
    pub success: bool,
}

pub static TOKEN_TIMEOUT_SECS: u64 = 30;

impl Claims {
    pub fn new(response: ResponseClaims) -> Self {
        Self::with_timeout(Duration::from_secs(TOKEN_TIMEOUT_SECS), response)
    }

    pub fn with_timeout(timeout: Duration, response: ResponseClaims) -> Self {
        Self {
            exp: OffsetDateTime::now_utc() + timeout,
            custom: response,
        }
    }
}
