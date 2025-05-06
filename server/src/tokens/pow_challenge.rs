use std::time::Duration;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::analysis::proof_of_work::PowChallenge;

use super::TimeClaims;

pub static JWT_POW_ALGORITHM: Algorithm = Algorithm::HS256;

pub fn encode(
    pow_challenge: PowChallenge,
    enc_key_b64: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::encode(
        &Header::new(JWT_POW_ALGORITHM),
        &TimeClaims::with_timeout(Duration::from_secs(300), pow_challenge),
        &EncodingKey::from_base64_secret(enc_key_b64)?,
    )
}

pub fn encode_with_timeout(
    pow_challenge: PowChallenge,
    enc_key_b64: &str,
    timeout: Duration,
) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::encode(
        &Header::new(JWT_POW_ALGORITHM),
        &TimeClaims::with_timeout(timeout, pow_challenge),
        &EncodingKey::from_base64_secret(enc_key_b64)?,
    )
}

pub fn decode(jwt: &str, dec_key_b64: &str) -> Result<PowChallenge, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(JWT_POW_ALGORITHM);
    TimeClaims::<PowChallenge>::build_validation(&mut validation);

    jsonwebtoken::decode::<_>(
        jwt,
        &DecodingKey::from_base64_secret(dec_key_b64)?,
        &validation,
    )
    .map(|tok| tok.claims)
}
