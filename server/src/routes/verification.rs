use std::{collections::HashMap, fmt::Display, net::IpAddr, str::FromStr, sync::Arc};

use anyhow::Context;
use axum::{extract::State, Form, Json};
use axum_extra::extract::WithRejection;
use jsonwebtoken::errors::ErrorKind;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use tracing::{instrument, Level};
use url::Host;

use super::errors::VerificationError;
use crate::{db, response_token, AppState};

#[derive(Debug)]
pub struct VerificationRequest {
    secret: Secret<String>,
    response: String,
    remoteip: Option<IpAddr>,
}

#[derive(Debug, Serialize, Deserialize, Error)]
pub struct VerificationResponse {
    pub success: bool,
    #[serde(with = "time::serde::iso8601")]
    pub challenge_ts: OffsetDateTime,
    #[serde(with = "crate::serde::option_host_as_str")]
    pub hostname: Option<Host>,
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

#[instrument(skip(state), err(Debug, level = Level::DEBUG))]
pub async fn site_verify(
    State(state): State<Arc<AppState>>,
    WithRejection(Form(verification), _): WithRejection<
        Form<HashMap<String, String>>,
        VerificationError,
    >,
) -> Result<Json<VerificationResponse>, VerificationError> {
    let verification: Result<VerificationRequest, Vec<ErrorCodes>> = verification.try_into();
    let verification = verification.map_err(VerificationResponse::failure)?;

    let enc_key = db::fetch_api_key_by_secret(&state.pool, verification.secret.expose_secret())
        .await
        .context("failed to fetch encoding key bey api secret while verifying challenge")?
        .ok_or(VerificationResponse::failure(vec![
            ErrorCodes::InvalidInputSecret,
        ]))?
        .encoding_key;

    let claims = response_token::decode(&verification.response, &enc_key)
        .map_err(|err| match err.into_kind() {
            ErrorKind::ExpiredSignature => ErrorCodes::TimeoutOrDuplicate,
            _ => ErrorCodes::InvalidInputResponse,
        })
        .map_err(|err_code| VerificationResponse::failure(vec![err_code]))?;

    let solver_check = verification
        .remoteip
        .map_or(true, |solver| solver == claims.custom.ip_addr);

    Ok(Json(VerificationResponse {
        success: claims.custom.score >= 0.5 && solver_check,
        challenge_ts: *claims.iat(),
        hostname: Host::parse(&claims.custom.hostname.to_string()).ok(),
        error_codes: None,
    }))
}

impl VerificationResponse {
    pub fn failure(errors: Vec<ErrorCodes>) -> Self {
        Self {
            success: false,
            challenge_ts: OffsetDateTime::UNIX_EPOCH,
            hostname: None,
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
        let remoteip = form.remove("remoteip").as_deref().and_then(|ip| {
            IpAddr::from_str(ip).ok().or_else(|| {
                errors.push(ErrorCodes::BadRequest);
                None
            })
        });
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
            remoteip,
        })
    }
}

impl Display for VerificationResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "verification: challenge loaded at {} in `{:?}` - {:?}",
            self.challenge_ts, self.hostname, self.error_codes
        )
    }
}
