use std::{collections::HashMap, fmt::Display};

use axum::{Form, Json};
use axum_extra::extract::WithRejection;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use tracing::instrument;

use crate::response_token;

use super::errors::VerificationError;

#[derive(Debug)]
pub struct VerificationRequest {
    secret: Secret<String>,
    response: String,
    #[expect(dead_code)]
    remoteip: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Error)]
pub struct VerificationResponse {
    pub success: bool,
    #[serde(with = "time::serde::iso8601")]
    pub challenge_ts: OffsetDateTime,
    pub hostname: String,
    #[serde(rename = "error-codes", skip_serializing_if = "Option::is_none")]
    pub error_codes: Option<Vec<ErrorCodes>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCodes {
    MissingInputSecret,
    InvalidInputSecret,
    MissingInputResponse,
    InvalidInputResponse,
    BadRequest,
    TimeoutOrDuplicate,
}

#[instrument]
pub async fn site_verify(
    WithRejection(Form(verification), _): WithRejection<
        Form<HashMap<String, String>>,
        VerificationError,
    >,
) -> Result<Json<VerificationResponse>, VerificationError> {
    let verification: Result<VerificationRequest, Vec<ErrorCodes>> = verification.try_into();
    // TODO: fetch from DB
    let (challenge_ts, hostname) = (OffsetDateTime::now_utc(), "unknown".to_string());
    let verification = verification
        .map_err(|errs| VerificationResponse::failure(challenge_ts, hostname.clone(), errs))?;

    // TODO: actually validate token
    if !valid_secret(verification.secret.expose_secret())? {
        return Err(VerificationError::UserError(VerificationResponse::failure(
            challenge_ts,
            hostname.clone(),
            vec![ErrorCodes::InvalidInputSecret],
        )));
    }

    let claims = response_token::decode(&verification.response)
        .map_err(jsonwebtoken::errors::Error::into_kind)
        .map_err(|err| match err {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => ErrorCodes::TimeoutOrDuplicate,
            _ => ErrorCodes::InvalidInputResponse,
        })
        .map_err(|err_code| {
            VerificationError::UserError(VerificationResponse::failure(
                challenge_ts,
                hostname.clone(),
                vec![err_code],
            ))
        })?;

    // TODO: check database to see if it's a duplicate

    Ok(Json(VerificationResponse {
        success: claims.success,
        challenge_ts,
        hostname,
        error_codes: None,
    }))
}

fn valid_secret(secret: &str) -> anyhow::Result<bool> {
    Ok(!secret.is_empty())
}

impl VerificationResponse {
    pub fn failure(
        challenge_ts: OffsetDateTime,
        hostname: String,
        errors: Vec<ErrorCodes>,
    ) -> Self {
        Self {
            success: false,
            challenge_ts,
            hostname,
            error_codes: Some(errors),
        }
    }
}

impl TryFrom<HashMap<String, String>> for VerificationRequest {
    type Error = Vec<ErrorCodes>;

    fn try_from(mut form: HashMap<String, String>) -> Result<Self, Self::Error> {
        let mut errors = vec![];
        if !form.contains_key("secret") {
            errors.push(ErrorCodes::MissingInputSecret);
        }
        if !form.contains_key("response") {
            errors.push(ErrorCodes::MissingInputResponse);
        }
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(VerificationRequest {
            secret: Secret::new(
                form.remove("secret")
                    .expect("checked if it contains key before"),
            ),
            response: form
                .remove("response")
                .expect("checked if it contains key before"),
            remoteip: form.remove("remoteip"),
        })
    }
}

impl Display for VerificationResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "verification: callenge loaded at {} in `{}` - {:?}",
            self.challenge_ts, self.hostname, self.error_codes
        )
    }
}
