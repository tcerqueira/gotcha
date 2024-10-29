mod errors;
pub mod internal;
pub mod public;

type Result<T> = std::result::Result<T, errors::Error>;
