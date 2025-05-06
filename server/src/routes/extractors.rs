use std::sync::Arc;

#[cfg(feature = "aws-lambda")]
use axum::http::Request;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

#[cfg(feature = "aws-lambda")]
pub fn extract_lambda_source_ip<B>(mut request: Request<B>) -> Request<B> {
    use axum::extract::ConnectInfo;
    use lambda_http::{RequestExt, request::RequestContext};
    use std::net::{IpAddr, SocketAddr};

    if request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .is_some()
    {
        return request;
    }

    let Some(RequestContext::ApiGatewayV2(cx)) = request.request_context_ref() else {
        tracing::error!("lambda context (ApiGatewayV2) not found in request");
        return request;
    };

    let Some(source_ip) = &cx.http.source_ip else {
        tracing::error!("source_ip not found in lambda context (http)");
        return request;
    };

    match source_ip.parse::<IpAddr>() {
        Ok(ip) => {
            request
                .extensions_mut()
                .insert(ConnectInfo(SocketAddr::new(ip, 443)));
        }
        Err(e) => tracing::error!(source_ip, err = ?e, "could not parse source_ip from request"),
    };

    request
}

#[cfg(feature = "aws-lambda")]
pub fn extract_lambda_origin<B>(mut request: Request<B>) -> Request<B> {
    pub use lambda_http::{RequestExt, request::RequestContext};

    let Some(RequestContext::ApiGatewayV2(cx)) = request.request_context_ref() else {
        tracing::error!("lambda context (ApiGatewayV2) not found in request");
        return request;
    };
    let Some(ref domain) = cx.domain_name else {
        tracing::error!("domain name not found in request");
        return request;
    };
    let origin = format!("https://{domain}");

    request.extensions_mut().insert(ThisOrigin(origin));
    request
}

#[derive(Debug, Clone)]
pub struct ThisOrigin(pub String);

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
                tracing::error!("could not extract origin");
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub user_id: Arc<str>,
}

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
