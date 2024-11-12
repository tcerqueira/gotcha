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
    #[error(transparent)]
    Challenge(#[from] ChallengeError),
    #[error(transparent)]
    Console(#[from] ConsoleError),
    #[error(transparent)]
    Verification(#[from] VerificationError),
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Unexpected(_) | Error::Sql(_) => {
                tracing::error!(error = ?self, "Internal Server Error ocurred.");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Error::Challenge(err) => err.into_response(),
            Error::Console(err) => err.into_response(),
            Error::Verification(err) => err.into_response(),
        }
    }
}

impl From<VerificationResponse> for Error {
    fn from(value: VerificationResponse) -> Self {
        Error::Verification(VerificationError::UserError(value))
    }
}

#[derive(Debug, Error)]
pub enum ChallengeError {
    #[error("Invalid secret, check your allowed-origins")]
    InvalidSecret,
}

impl IntoResponse for ChallengeError {
    fn into_response(self) -> Response {
        match self {
            ChallengeError::InvalidSecret => {
                (StatusCode::FORBIDDEN, self.to_string()).into_response()
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum ConsoleError {
    #[error("Not found: {what}")]
    NotFound { what: String },
    #[error("Access forbidden")]
    Forbidden,
}

impl IntoResponse for ConsoleError {
    fn into_response(self) -> Response {
        match self {
            ConsoleError::NotFound { what } => (StatusCode::NOT_FOUND, what).into_response(),
            ConsoleError::Forbidden => StatusCode::FORBIDDEN.into_response(),
        }
    }
}

#[derive(Debug, Error)]
pub enum VerificationError {
    #[error(transparent)]
    UserError(#[from] VerificationResponse),
    #[error(transparent)]
    BadRequest(#[from] FormRejection),
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
        }
    }
}
