use std::sync::Arc;

use admin::{add_challenge, remove_challenge};
use axum::{
    routing::{delete, get, post},
    Router,
};
use challenge::{get_challenge, process_challenge};
use console::{add_origin, create_console, gen_api_secret, remove_origin};
use middleware::require_auth_mw;
use public::site_verify;

use crate::AppState;

pub mod admin;
pub mod challenge;
pub mod console;
mod errors;
pub mod middleware;
pub mod public;

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
        .route("/", post(create_console))
        .route("/secret", post(gen_api_secret))
        .route("/origin", post(add_origin))
        .route("/origin", delete(remove_origin))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            require_auth_mw,
        ))
        .with_state(state)
}

pub fn admin(state: &Arc<AppState>) -> Router {
    let state = Arc::clone(state);
    Router::new()
        .route("/challenge", post(add_challenge))
        .route("/challenge", delete(remove_challenge))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            require_auth_mw,
        ))
        .with_state(state)
}
