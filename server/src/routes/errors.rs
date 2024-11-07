use axum::{
    extract::rejection::FormRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;
use time::OffsetDateTime;

use super::public::{ErrorCodes, VerificationResponse};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid secret, check your allowed-origins")]
    InvalidSecret,
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Unexpected(_) | Error::Sql(_) => {
                tracing::error!(error = %self, "Internal server error ocurred");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Error::InvalidSecret => (StatusCode::FORBIDDEN, self.to_string()).into_response(),
        }
    }
}

#[derive(Debug, Error)]
pub enum VerificationError {
    #[error(transparent)]
    UserError(#[from] VerificationResponse),
    #[error(transparent)]
    BadRequest(#[from] FormRejection),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for VerificationError {
    fn into_response(self) -> Response {
        match self {
            VerificationError::UserError(verification) => Json(verification).into_response(),
            VerificationError::BadRequest(_) => Json(VerificationResponse::failure(
                OffsetDateTime::UNIX_EPOCH,
                "".to_string(),
                vec![ErrorCodes::BadRequest],
            ))
            .into_response(),
            VerificationError::UnexpectedError(error) => {
                tracing::error!("Internal Server Error (500): {}", error);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        }
    }
}
