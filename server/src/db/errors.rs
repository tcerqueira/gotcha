use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Row not found")]
    NotFound,
    #[error("{kind} constraint not met: {source}")]
    Constraint { source: Box<PgDatabaseError>, kind: ConstraintKind },
    #[error(transparent)]
    Other(sqlx::Error),
}

#[derive(Debug, Error)]
pub enum ConstraintKind {
    #[error("Primary key")]
    PrimaryKey,
    #[error("Foreign key")]
    ForeignKey,
    #[error("Unique key")]
    UniqueKey,
    #[error("Positive width and height")]
    DimensionsPositive,
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
                    ref err
                        if err
                            .constraint()
                            .is_some_and(|c| c == "width_positive" || c == "height_positive") =>
                    {
                        Self::Constraint {
                            source: pg_err,
                            kind: ConstraintKind::DimensionsPositive,
                        }
                    }
                    _ => Self::Other(sqlx::Error::from(*pg_err)),
                }
            }
            e => Self::Other(e),
        }
    }
}
