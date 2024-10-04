use axum::{Form, Json};
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
    Form(_verification): Form<VerificationRequest>,
) -> Json<VerificationResponse> {
    todo!()
}
