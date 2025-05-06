use std::{collections::HashSet, time::Duration};

use jsonwebtoken::Validation;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;

pub mod auth;
pub mod pow_challenge;
pub mod response;

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeClaims<T> {
    #[serde(with = "time::serde::timestamp")]
    exp: OffsetDateTime,
    #[serde(with = "time::serde::timestamp")]
    iat: OffsetDateTime,
    #[serde(flatten)]
    pub other: T,
}

impl<T> TimeClaims<T> {
    pub const TIMEOUT_SECS: u64 = 30;

    pub fn new(other_claims: T) -> Self {
        Self::with_timeout(Duration::from_secs(Self::TIMEOUT_SECS), other_claims)
    }

    pub fn with_timeout(timeout: Duration, other_claims: T) -> Self {
        let now = OffsetDateTime::now_utc();
        Self { exp: now + timeout, iat: now, other: other_claims }
    }

    pub fn build_validation(validation: &mut Validation) {
        validation.required_spec_claims.insert("exp".into());
        validation.validate_exp = true;
        validation.leeway = 0;
    }

    pub fn exp(&self) -> &OffsetDateTime {
        &self.exp
    }

    pub fn iat(&self) -> &OffsetDateTime {
        &self.iat
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims<T> {
    #[serde(
        with = "crate::serde::single_or_sequence",
        skip_serializing_if = "Vec::is_empty"
    )]
    aud: Vec<Url>,
    sub: String,
    iss: String,
    #[serde(flatten)]
    pub other: T,
}

impl<T> AuthClaims<T> {
    pub fn build_validation(validation: &mut Validation) {
        validation
            .required_spec_claims
            .extend(["aud".into(), "iss".into(), "sub".into()]);
        validation
            .aud
            .get_or_insert_with(HashSet::default)
            .insert("https://gotcha.land/".into());
        validation.validate_aud = true;
    }

    pub fn sub(&self) -> &str {
        &self.sub
    }

    pub fn iss(&self) -> &str {
        &self.iss
    }

    pub fn aud(&self) -> &[Url] {
        &self.aud
    }
}
