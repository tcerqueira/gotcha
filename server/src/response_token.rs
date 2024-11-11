use std::time::Duration;

use anyhow::Context;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::routes::challenge::{Claims, ResponseClaims};

pub static JWT_ALGORITHM: Algorithm = Algorithm::HS256;

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

pub fn decode(jwt: &str, dec_key_b64: &str) -> Result<ResponseClaims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.leeway = 0;

    jsonwebtoken::decode::<ResponseClaims>(
        jwt,
        &DecodingKey::from_base64_secret(dec_key_b64)?,
        &validation,
    )
    .map(|data| data.claims)
}
