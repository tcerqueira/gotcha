use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::{Form, Json};
use reqwest::Url;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{AppState, Challenge};

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
pub struct VerificationRequest {
    secret: Secret<String>,
    response: String,
    remoteip: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VerificationResponse {
    success: bool,
    #[serde(with = "time::serde::iso8601")]
    challenge_ts: OffsetDateTime,
    hostname: String,
    #[serde(rename = "error-codes")]
    error_codes: Option<Vec<ErrorCodes>>,
}

#[expect(dead_code)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCodes {
    MissingInputSecret,
    InvalidInputSecret,
    MissingInputResponse,
    InvalidInputResponse,
    BadRequest,
    TimeoutOrDuplicate,
}

pub async fn site_verify(
    Form(verification): Form<HashMap<String, String>>,
) -> Json<VerificationResponse> {
    tracing::info!("{verification:?}");

    Json(VerificationResponse {
        success: false,
        challenge_ts: OffsetDateTime::now_utc(),
        hostname: "wtv".into(),
        error_codes: None,
    })
}

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

pub async fn process_challenge() {
    todo!()
}
