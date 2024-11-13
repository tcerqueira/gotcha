use std::{collections::HashMap, fmt::Display, sync::Arc};

use anyhow::Context;
use axum::{extract::State, Form, Json};
use axum_extra::extract::WithRejection;
use jsonwebtoken::errors::ErrorKind;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use tracing::instrument;

use crate::{db, response_token, AppState};

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

#[instrument(skip(state))]
pub async fn site_verify(
    State(state): State<Arc<AppState>>,
    WithRejection(Form(verification), _): WithRejection<
        Form<HashMap<String, String>>,
        VerificationError,
    >,
) -> super::Result<Json<VerificationResponse>> {
    let verification: Result<VerificationRequest, Vec<ErrorCodes>> = verification.try_into();

    let verification = verification.map_err(VerificationResponse::failure)?;

    let enc_key = db::fetch_encoding_key(&state.pool, verification.secret.expose_secret())
        .await
        .context("failed to fetch encoding key bey api secret while verifying challenge")?
        .ok_or(VerificationResponse::failure(vec![
            ErrorCodes::InvalidInputSecret,
        ]))?;

    let claims = response_token::decode(&verification.response, &enc_key)
        .map_err(|err| match err.into_kind() {
            ErrorKind::ExpiredSignature => ErrorCodes::TimeoutOrDuplicate,
            _ => ErrorCodes::InvalidInputResponse,
        })
        .map_err(|err_code| VerificationResponse::failure(vec![err_code]))?;

    Ok(Json(VerificationResponse {
        success: claims.custom.success,
        challenge_ts: claims.iat(),
        // TODO: add 'sub' or custom claim
        hostname: "".into(),
        error_codes: None,
    }))
}

impl VerificationResponse {
    pub fn failure(errors: Vec<ErrorCodes>) -> Self {
        Self {
            success: false,
            challenge_ts: OffsetDateTime::UNIX_EPOCH,
            hostname: "".into(),
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
            "verification: challenge loaded at {} in `{}` - {:?}",
            self.challenge_ts, self.hostname, self.error_codes
        )
    }
}
