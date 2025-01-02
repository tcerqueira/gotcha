use std::sync::Arc;

use admin::{add_challenge, remove_challenge};
use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use challenge::{get_challenge, process_challenge};
use console::{
    create_console, delete_console, gen_api_key, get_api_keys, get_consoles, revoke_api_key,
    update_console,
};
use middleware::{require_admin, require_auth, validate_console_id};
use verification::site_verify;

use crate::AppState;

pub mod admin;
pub mod challenge;
pub mod console;
mod errors;
pub mod middleware;
pub mod verification;

pub fn challenge(state: &Arc<AppState>) -> Router {
    let state = Arc::clone(state);
    Router::new()
        .route("/", get(get_challenge))
        .route("/process", post(process_challenge))
        .with_state(state)
}

pub fn verification(state: &Arc<AppState>) -> Router {
    let state = Arc::clone(state);
    Router::new()
        .route("/siteverify", post(site_verify))
        .with_state(state)
}

pub fn console(state: &Arc<AppState>) -> Router {
    let state = Arc::clone(state);
    Router::new()
        .route("/", get(get_consoles))
        .route("/", post(create_console))
        .nest(
            "/:console_id",
            Router::new()
                .route("/", patch(update_console))
                .route("/", delete(delete_console))
                .route("/api-key", get(get_api_keys))
                .route("/api-key", post(gen_api_key))
                .route("/api-key/:site_key", delete(revoke_api_key))
                .layer(axum::middleware::from_fn_with_state(
                    Arc::clone(&state),
                    validate_console_id,
                )),
        )
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            require_auth,
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
            require_admin,
        ))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            require_auth,
        ))
        .with_state(state)
}
