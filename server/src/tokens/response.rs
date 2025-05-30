use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, time::Duration};
use url::Host;

use super::TimeClaims;

pub static JWT_RESPONSE_ALGORITHM: Algorithm = Algorithm::HS256;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseClaims {
    pub score: f32,
    pub addr: IpAddr,
    #[serde(with = "crate::serde::host_as_str")]
    pub host: Host,
}

pub fn encode(
    response_claims: ResponseClaims,
    enc_key_b64: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::encode(
        &Header::new(JWT_RESPONSE_ALGORITHM),
        &TimeClaims::new(response_claims),
        &EncodingKey::from_base64_secret(enc_key_b64)?,
    )
}

pub fn encode_with_timeout(
    response_claims: ResponseClaims,
    enc_key_b64: &str,
    timeout: Duration,
) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::encode(
        &Header::new(JWT_RESPONSE_ALGORITHM),
        &TimeClaims::with_timeout(timeout, response_claims),
        &EncodingKey::from_base64_secret(enc_key_b64)?,
    )
}

pub fn decode(
    jwt: &str,
    dec_key_b64: &str,
) -> Result<TimeClaims<ResponseClaims>, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(JWT_RESPONSE_ALGORITHM);
    TimeClaims::<ResponseClaims>::build_validation(&mut validation);

    jsonwebtoken::decode::<TimeClaims<_>>(
        jwt,
        &DecodingKey::from_base64_secret(dec_key_b64)?,
        &validation,
    )
    .map(|tok| tok.claims)
}
