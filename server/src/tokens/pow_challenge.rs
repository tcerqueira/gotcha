use std::time::Duration;

use jsonwebtoken::Algorithm;

use super::Claims;

use crate::analysis::proof_of_work::PowChallenge;

pub static JWT_POW_ALGORITHM: Algorithm = Algorithm::HS256;

pub fn encode(
    pow_challenge: PowChallenge,
    enc_key_b64: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    super::encode_with_timeout(
        JWT_POW_ALGORITHM,
        pow_challenge,
        enc_key_b64,
        Duration::from_secs(300),
    )
}

pub fn encode_with_timeout(
    pow_challenge: PowChallenge,
    enc_key_b64: &str,
    timeout: Duration,
) -> Result<String, jsonwebtoken::errors::Error> {
    super::encode_with_timeout(JWT_POW_ALGORITHM, pow_challenge, enc_key_b64, timeout)
}

pub fn decode(
    jwt: &str,
    dec_key_b64: &str,
) -> Result<Claims<PowChallenge>, jsonwebtoken::errors::Error> {
    super::decode(JWT_POW_ALGORITHM, jwt, dec_key_b64)
}
