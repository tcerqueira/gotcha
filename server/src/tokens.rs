use std::time::Duration;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use time::OffsetDateTime;

pub mod response;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims<T> {
    #[serde(with = "time::serde::timestamp")]
    exp: OffsetDateTime,
    #[serde(with = "time::serde::timestamp")]
    iat: OffsetDateTime,
    #[serde(flatten)]
    pub custom: T,
}

fn encode<T: Serialize>(
    alg: Algorithm,
    custom_claims: T,
    enc_key_b64: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::encode(
        &Header::new(alg),
        &Claims::new(custom_claims),
        &EncodingKey::from_base64_secret(enc_key_b64)?,
    )
}

fn encode_with_timeout<T: Serialize>(
    alg: Algorithm,
    custom_claims: T,
    enc_key_b64: &str,
    timeout: Duration,
) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::encode(
        &Header::new(alg),
        &Claims::with_timeout(timeout, custom_claims),
        &EncodingKey::from_base64_secret(enc_key_b64)?,
    )
}

fn decode<T: DeserializeOwned>(
    alg: Algorithm,
    jwt: &str,
    dec_key_b64: &str,
) -> Result<Claims<T>, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(alg);
    validation.leeway = 0;

    jsonwebtoken::decode::<Claims<T>>(
        jwt,
        &DecodingKey::from_base64_secret(dec_key_b64)?,
        &validation,
    )
    .map(|tok| tok.claims)
}

impl<T> Claims<T> {
    pub const TIMEOUT_SECS: u64 = 30;

    pub fn new(custom_claims: T) -> Self {
        Self::with_timeout(Duration::from_secs(Self::TIMEOUT_SECS), custom_claims)
    }

    pub fn with_timeout(timeout: Duration, custom_claims: T) -> Self {
        let now = OffsetDateTime::now_utc();
        Self { exp: now + timeout, iat: now, custom: custom_claims }
    }

    pub fn exp(&self) -> &OffsetDateTime {
        &self.exp
    }

    pub fn iat(&self) -> &OffsetDateTime {
        &self.iat
    }
}
