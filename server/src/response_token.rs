use std::{net::SocketAddr, time::Duration};

use anyhow::Context;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub static JWT_ALGORITHM: Algorithm = Algorithm::HS256;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    #[serde(with = "time::serde::timestamp")]
    exp: OffsetDateTime,
    #[serde(with = "time::serde::timestamp")]
    iat: OffsetDateTime,
    #[serde(flatten)]
    pub custom: ResponseClaims,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseClaims {
    pub success: bool,
    pub authority: SocketAddr,
}

pub fn encode(response_claims: ResponseClaims, enc_key_b64: &str) -> anyhow::Result<String> {
    jsonwebtoken::encode(
        &Header::new(JWT_ALGORITHM),
        &Claims::new(response_claims),
        &EncodingKey::from_base64_secret(enc_key_b64).context("invalid secret")?,
    )
    .context("failed encoding to jwt")
}

pub fn encode_with_timeout(
    timeout: Duration,
    response_claims: ResponseClaims,
    enc_key_b64: &str,
) -> anyhow::Result<String> {
    jsonwebtoken::encode(
        &Header::new(JWT_ALGORITHM),
        &Claims::with_timeout(timeout, response_claims),
        &EncodingKey::from_base64_secret(enc_key_b64).context("invalid secret")?,
    )
    .context("failed encoding to jwt")
}

pub fn decode(jwt: &str, dec_key_b64: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.leeway = 0;

    jsonwebtoken::decode::<Claims>(
        jwt,
        &DecodingKey::from_base64_secret(dec_key_b64)?,
        &validation,
    )
    .map(|data| data.claims)
}

impl Claims {
    pub const TIMEOUT_SECS: u64 = 30;

    pub fn new(response: ResponseClaims) -> Self {
        Self::with_timeout(Duration::from_secs(Self::TIMEOUT_SECS), response)
    }

    pub fn with_timeout(timeout: Duration, response: ResponseClaims) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            exp: now + timeout,
            iat: now,
            custom: response,
        }
    }

    pub fn exp(&self) -> OffsetDateTime {
        self.exp
    }

    pub fn iat(&self) -> OffsetDateTime {
        self.iat
    }
}
