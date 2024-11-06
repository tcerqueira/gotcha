pub mod challenge;
mod errors;
pub mod public;

type Result<T> = std::result::Result<T, errors::Error>;
