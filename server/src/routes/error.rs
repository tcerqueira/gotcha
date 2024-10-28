use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Unexpected(err) => {
                tracing::error!("internal server error (500): {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected Server Error")
            }
        }
        .into_response()
    }
}
