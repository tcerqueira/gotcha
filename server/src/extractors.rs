use std::sync::Arc;

use anyhow::Context;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{async_trait, extract::FromRequestParts};

#[cfg(feature = "aws-lambda")]
mod aws_lambda {
    pub use axum::{extract::ConnectInfo, http::Request};
    pub use lambda_http::{request::RequestContext, RequestExt};
    pub use std::net::{IpAddr, SocketAddr};
}
#[cfg(feature = "aws-lambda")]
pub use aws_lambda::*;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::typed_header::TypedHeaderRejection;
use axum_extra::TypedHeader;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{DecodingKey, Validation};
use serde::Deserialize;
use thiserror::Error;

use crate::{AppState, HTTP_CACHE_CLIENT};

#[cfg(feature = "aws-lambda")]
pub fn extract_lambda_source_ip<B>(mut request: Request<B>) -> Request<B> {
    if request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .is_some()
    {
        return request;
    }

    let Some(RequestContext::ApiGatewayV2(cx)) = request.request_context_ref() else {
        return request;
    };

    let Some(source_ip) = &cx.http.source_ip else {
        return request;
    };

    if let Ok(ip) = source_ip.parse::<IpAddr>() {
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::new(ip, 443)));
    } else {
        tracing::error!(source_ip, "Could not parse source_ip from request");
    }

    request
}

#[cfg(feature = "aws-lambda")]
pub fn extract_lambda_origin<B>(mut request: Request<B>) -> Request<B> {
    let Some(RequestContext::ApiGatewayV2(cx)) = request.request_context_ref() else {
        return request;
    };
    let Some(ref domain) = cx.domain_name else {
        tracing::error!("Domain name not found in request");
        return request;
    };
    let origin = format!("https://{domain}");

    request.extensions_mut().insert(ThisOrigin(origin));
    request
}

#[derive(Debug, Clone)]
pub struct ThisOrigin(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for ThisOrigin
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<ThisOrigin>()
            .cloned()
            .ok_or_else(|| {
                tracing::error!("Could not extract origin");
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }
}

#[derive(Debug, Clone)]
pub struct User(pub String);

#[async_trait]
impl FromRequestParts<Arc<AppState>> for User {
    type Rejection = UserRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await?;
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

        #[derive(Debug, Deserialize)]
        struct AuthClaims {
            sub: String,
        }

        let claims = jsonwebtoken::decode::<AuthClaims>(
            token,
            &DecodingKey::from_jwk(
                jwks.find(&kid)
                    .with_context(|| format!("kid {} not found in jwks", kid))?,
            )
            .context("could not create decoding key from JWK")?,
            &validation,
        )?;

        Ok(User(claims.claims.sub))
    }
}

#[derive(Debug, Error)]
pub enum UserRejection {
    #[error("Bad header: {0}")]
    Header(#[from] TypedHeaderRejection),
    #[error("Could not retrieve JWK set: {0}")]
    Jwk(#[from] reqwest_middleware::Error),
    #[error("Invalid JWT: {0}")]
    Token(#[from] jsonwebtoken::errors::Error),
    #[error("Other: {0}")]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for UserRejection {
    fn into_response(self) -> axum::response::Response {
        tracing::debug!("UserRejection into_response: {:?}", self);
        match self {
            UserRejection::Header(err) => err.into_response(),
            UserRejection::Jwk(err) => {
                tracing::error!("Could not retrieve JWK set: {:?}", err);
                match err.is_decode() {
                    true => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
                    false => (StatusCode::SERVICE_UNAVAILABLE).into_response(),
                }
            }
            UserRejection::Token(_) | UserRejection::Other(_) => {
                (StatusCode::FORBIDDEN).into_response()
            }
        }
    }
}

impl From<reqwest::Error> for UserRejection {
    fn from(value: reqwest::Error) -> Self {
        UserRejection::Jwk(reqwest_middleware::Error::from(value))
    }
}
