use std::collections::HashMap;

use axum::extract::Query;
use axum::{Form, Json};
use reqwest::Url;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

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

#[derive(Debug, Serialize)]
pub struct Challenge {
    url: String,
    width: u16,
    height: u16,
}

pub async fn get_challenge(Query(params): Query<HashMap<String, String>>) -> Json<Challenge> {
    let challenges = [Challenge {
        url: String::from("http://localhost:8080/im-not-a-robot/index.html"),
        width: 304,
        height: 78,
    }];

    let mut url = Url::parse(&challenges[0].url).unwrap();
    url.query_pairs_mut().append_pair("token", &params["token"]);

    Json(Challenge {
        url: url.to_string(),
        ..challenges[0]
    })
}

pub async fn process_challenge() {
    todo!()
}
