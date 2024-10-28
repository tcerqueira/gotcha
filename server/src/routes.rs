mod error;
pub mod internal;
pub mod public;

type Result<T> = std::result::Result<T, error::Error>;
