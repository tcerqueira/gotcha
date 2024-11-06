use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};
use challenge::{get_challenge, process_challenge};
use console::{add_origin, gen_api_secret, remove_origin};
use public::site_verify;

use crate::AppState;

pub mod challenge;
pub mod console;
mod errors;
pub mod public;

type Result<T> = std::result::Result<T, errors::Error>;

pub fn challenge(state: &Arc<AppState>) -> Router {
    let state = Arc::clone(state);
    Router::new()
        .route("/", get(get_challenge))
        .route("/process", post(process_challenge))
        .with_state(state)
}

pub fn public(state: &Arc<AppState>) -> Router {
    let state = Arc::clone(state);
    Router::new()
        .route("/siteverify", post(site_verify))
        .with_state(state)
}

pub fn console(state: &Arc<AppState>) -> Router {
    let state = Arc::clone(state);
    Router::new()
        .route("/api-secret", post(gen_api_secret))
        .route("/allowed-origin", post(add_origin))
        .route("/allowed-origin", delete(remove_origin))
        .with_state(state)
}
