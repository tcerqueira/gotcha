use axum::{
    extract::rejection::FormRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::typed_header::TypedHeaderRejection;
use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

use super::public::{ErrorCodes, VerificationResponse};

#[derive(Debug, Error)]
pub enum ChallengeError {
    #[error("Invalid secret")]
    InvalidSecret,
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for ChallengeError {
    fn into_response(self) -> Response {
        match self {
            ChallengeError::Unexpected(_) | ChallengeError::Sql(_) => {
                tracing::error!(error = ?self, "Internal Server Error ocurred.");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
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
    #[error("Duplicate")]
    Duplicate,
    #[error(transparent)]
    Sql(sqlx::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for ConsoleError {
    fn into_response(self) -> Response {
        match self {
            ConsoleError::Unexpected(_) | ConsoleError::Sql(_) | ConsoleError::Duplicate => {
                tracing::error!(error = ?self, "Internal Server Error ocurred.");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            ConsoleError::NotFound { what: _ } => {
                (StatusCode::NOT_FOUND, self.to_string()).into_response()
            }
            ConsoleError::Forbidden => StatusCode::FORBIDDEN.into_response(),
        }
    }
}

impl From<sqlx::Error> for ConsoleError {
    fn from(db_err: sqlx::Error) -> Self {
        match db_err {
            sqlx::Error::Database(err)
                if err
                    .downcast_ref::<PgDatabaseError>()
                    .constraint()
                    .is_some_and(|c| c == "api_key_console_id_fkey") =>
            {
                ConsoleError::Forbidden
            }
            sqlx::Error::Database(err)
                if err
                    .downcast_ref::<PgDatabaseError>()
                    .constraint()
                    .is_some_and(|c| c == "api_key_secret_unique" || c == "api_key_pkey") =>
            {
                ConsoleError::Duplicate
            }
            err => ConsoleError::Sql(err),
        }
    }
}

#[derive(Debug, Error)]
pub enum AdminError {
    #[error("{what} resource already exists")]
    NotUnique { what: String },
    #[error("Dimensions out of range: width and height must be greater than 0")]
    InvalidDimensions,
    #[error("Could not parse URL")]
    InvalidUrl,
    #[error("Challenge not found: url('{0}')")]
    NotFound(String),
    #[error(transparent)]
    Unauthorized(#[from] TypedHeaderRejection),
    #[error(transparent)]
    Sql(sqlx::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for AdminError {
    fn into_response(self) -> Response {
        match self {
            AdminError::Unexpected(_) | AdminError::Sql(_) => {
                tracing::error!(error = ?self, "Internal Server Error ocurred.");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            AdminError::NotUnique { what: _ } => {
                (StatusCode::CONFLICT, self.to_string()).into_response()
            }
            AdminError::InvalidDimensions => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response()
            }
            AdminError::InvalidUrl => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            AdminError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            AdminError::Unauthorized(err) => {
                (StatusCode::UNAUTHORIZED, err.to_string()).into_response()
            }
        }
    }
}

impl From<sqlx::Error> for AdminError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(db_err)
                if db_err
                    .downcast_ref::<PgDatabaseError>()
                    .constraint()
                    .is_some_and(|c| c == "width_positive" || c == "height_positive") =>
            {
                AdminError::InvalidDimensions
            }
            sqlx::Error::Database(db_err)
                if db_err
                    .downcast_ref::<PgDatabaseError>()
                    .constraint()
                    .is_some_and(|c| c == "challenge_pkey") =>
            {
                AdminError::NotUnique {
                    what: "Challenge url".into(),
                }
            }
            other => AdminError::Sql(other),
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
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for VerificationError {
    fn into_response(self) -> Response {
        match self {
            VerificationError::Unexpected(_) | VerificationError::Sql(_) => {
                tracing::error!(error = ?self, "Internal Server Error ocurred.");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            VerificationError::UserError(verification) => Json(verification).into_response(),
            VerificationError::BadRequest(_) => {
                Json(VerificationResponse::failure(vec![ErrorCodes::BadRequest])).into_response()
            }
        }
    }
}
