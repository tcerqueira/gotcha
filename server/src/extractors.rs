use std::sync::Arc;

use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::{async_trait, extract::FromRequestParts};

#[cfg(feature = "aws-lambda")]
mod aws_lambda {
    pub use axum::{extract::ConnectInfo, http::Request};
    pub use lambda_http::{request::RequestContext, RequestExt};
    pub use std::net::{IpAddr, SocketAddr};
}
#[cfg(feature = "aws-lambda")]
pub use aws_lambda::*;

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
pub struct User {
    pub user_id: Arc<str>,
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<User>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}
