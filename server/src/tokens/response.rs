use jsonwebtoken::Algorithm;
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, time::Duration};
use url::Host;

use super::Claims;

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
    super::encode(JWT_RESPONSE_ALGORITHM, response_claims, enc_key_b64)
}

pub fn encode_with_timeout(
    response_claims: ResponseClaims,
    enc_key_b64: &str,
    timeout: Duration,
) -> Result<String, jsonwebtoken::errors::Error> {
    super::encode_with_timeout(
        JWT_RESPONSE_ALGORITHM,
        response_claims,
        enc_key_b64,
        timeout,
    )
}

pub fn decode(
    jwt: &str,
    dec_key_b64: &str,
) -> Result<Claims<ResponseClaims>, jsonwebtoken::errors::Error> {
    super::decode(JWT_RESPONSE_ALGORITHM, jwt, dec_key_b64)
}
