use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Query, State};
use axum::Json;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{response_token, AppState, Challenge};

pub async fn get_challenge(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Challenge> {
    let challenge = &state.challenges[0];
    let mut url = Url::parse(&challenge.url).unwrap();
    url.query_pairs_mut().append_pair("token", &params["token"]);

    Json(Challenge {
        url: url.to_string(),
        ..*challenge
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResults {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub token: String,
}

pub static TOKEN_TIMEOUT_SECS: u64 = 30;

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
