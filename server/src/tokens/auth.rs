use jsonwebtoken::{Algorithm, DecodingKey, Validation};

use super::{AuthClaims, TimeClaims};

pub static JWT_AUTH_ALGORITHM: Algorithm = Algorithm::RS256;

pub fn decode(
    jwt: &str,
    dec_key: &DecodingKey,
) -> Result<AuthClaims<()>, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(JWT_AUTH_ALGORITHM);
    AuthClaims::<()>::build_validation(&mut validation);
    TimeClaims::<()>::build_validation(&mut validation);

    jsonwebtoken::decode::<AuthClaims<()>>(jwt, dec_key, &validation).map(|tok| tok.claims)
}
