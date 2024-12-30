use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{jwk::JwkSet, DecodingKey, Validation};
use serde::Deserialize;
use thiserror::Error;
use tracing::{instrument, Level};
use uuid::Uuid;

use crate::{db, extractors::User, AppState, HTTP_CACHE_CLIENT};

use super::errors::ConsoleError;

#[instrument(fields(user_id), skip(state, auth_header, request, next), err(Debug, level = Level::DEBUG))]
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    tracing::trace!(auth_header = ?auth_header);
    let token = auth_header.token();
    let header = jsonwebtoken::decode_header(token)?;
    let kid = header.kid.context("kid not present in header")?;

    let jwks: JwkSet = HTTP_CACHE_CLIENT
        .get(format!("{}/.well-known/jwks.json", state.auth_origin))
        .send()
        .await?
        .json()
        .await?;

    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&["https://console-rust-backend"]);
    validation.set_required_spec_claims(&["exp", "aud", "iss", "sub"]);

    let claims = jsonwebtoken::decode::<AuthClaims>(
        token,
        &DecodingKey::from_jwk(
            jwks.find(&kid)
                .with_context(|| format!("kid {} not found in jwks", kid))?,
        )
        .context("could not create decoding key from JWK")?,
        &validation,
    )?;
    tracing::Span::current().record("user_id", &claims.claims.sub);

    request.extensions_mut().insert(User {
        user_id: Arc::from(claims.claims.sub),
    });
    Ok(next.run(request).await)
}

#[derive(Debug, Deserialize)]
struct AuthClaims {
    sub: String,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Could not retrieve JWK set: {0}")]
    Jwk(#[from] reqwest_middleware::Error),
    #[error("Invalid JWT: {0}")]
    Token(#[from] jsonwebtoken::errors::Error),
    #[error("Other: {0}")]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AuthError::Jwk(err) => {
                tracing::error!("Could not retrieve JWK set: {:?}", err);
                match err.is_decode() {
                    true => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
                    false => (StatusCode::SERVICE_UNAVAILABLE).into_response(),
                }
            }
            AuthError::Token(_) | AuthError::Other(_) => (StatusCode::UNAUTHORIZED).into_response(),
        }
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(value: reqwest::Error) -> Self {
        AuthError::Jwk(reqwest_middleware::Error::from(value))
    }
}

#[derive(Debug, Deserialize)]
pub struct PathFields {
    pub console_id: Uuid,
}

#[instrument(skip_all, err(Debug, level = Level::DEBUG))]
pub async fn validate_console_id(
    State(state): State<Arc<AppState>>,
    Path(PathFields { console_id }): Path<PathFields>,
    User { user_id }: User,
    request: Request,
    next: Next,
) -> Result<Response, ConsoleError> {
    match db::exists_console_for_user(&state.pool, &console_id, &user_id).await? {
        true => Ok(next.run(request).await),
        false => Err(ConsoleError::Forbidden),
    }
}

#[instrument(skip_all)]
pub async fn require_admin(
    State(_state): State<Arc<AppState>>,
    User { user_id }: User,
    request: Request,
    next: Next,
) -> Response {
    match user_id.as_ref() {
        "Bk9vgyK6FiQ0oMHDT3b4EfQoIVRDs3ZM@clients" |    // dev
        "google-oauth2|106674402838515911816"           // tiago@bitfashioned.com
            => next.run(request).await,
        _ => StatusCode::FORBIDDEN.into_response(),
    }
}
