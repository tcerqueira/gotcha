//! Errors associated with the database.

use base64::DecodeError;
use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

/// High level kind of errors the database reports.
#[derive(Debug, Error)]
pub enum Error {
    #[error("{kind} constraint not met: {source}")]
    Constraint { source: Box<PgDatabaseError>, kind: ConstraintKind },
    #[error(transparent)]
    Other(sqlx::Error),
}

/// Kind of constraint errors.
#[derive(Debug, Error)]
pub enum ConstraintKind {
    #[error("Primary key")]
    PrimaryKey,
    #[error("Foreign key")]
    ForeignKey,
    #[error("Unique key")]
    UniqueKey,
    #[error("Value range")]
    ValueRange,
}

impl From<sqlx::Error> for Error {
    fn from(sql_err: sqlx::Error) -> Self {
        match sql_err {
            sqlx::Error::Database(err) => {
                let pg_err = err.downcast::<PgDatabaseError>();
                match pg_err {
                    ref err if err.constraint().is_some_and(|c| c.ends_with("_pkey")) => {
                        Self::Constraint { source: pg_err, kind: ConstraintKind::PrimaryKey }
                    }
                    ref err if err.constraint().is_some_and(|c| c.ends_with("_fkey")) => {
                        Self::Constraint { source: pg_err, kind: ConstraintKind::ForeignKey }
                    }
                    ref err if err.constraint().is_some_and(|c| c.ends_with("_unique")) => {
                        Self::Constraint { source: pg_err, kind: ConstraintKind::UniqueKey }
                    }
                    ref err if err.constraint().is_some_and(|c| c.ends_with("_range")) => {
                        Self::Constraint { source: pg_err, kind: ConstraintKind::ValueRange }
                    }
                    _ => Self::Other(sqlx::Error::from(*pg_err)),
                }
            }
            e => Self::Other(e),
        }
    }
}

impl From<DecodeError> for Error {
    fn from(value: DecodeError) -> Self {
        Self::Other(sqlx::Error::Decode(Box::new(value)))
    }
}
