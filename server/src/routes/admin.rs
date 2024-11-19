use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgDatabaseError;
use tracing::instrument;

use crate::{db, AppState};

use super::errors::AdminError;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddChallenge {
    pub url: String,
    pub width: u16,
    pub height: u16,
}

#[instrument(skip(state))]
pub async fn add_challenge(
    State(state): State<Arc<AppState>>,
    Json(challenge): Json<AddChallenge>,
) -> super::Result<()> {
    let AddChallenge { url, width, height } = challenge;
    let _ = Url::parse(&url).map_err(|_| AdminError::InvalidUrl)?;

    let res = db::insert_challenge(
        &state.pool,
        &db::DbChallenge {
            url,
            width: width as i16,
            height: height as i16,
        },
    )
    .await;

    if let Err(err) = res {
        return match err {
            sqlx::Error::Database(db_err)
                if db_err
                    .downcast_ref::<PgDatabaseError>()
                    .constraint()
                    .is_some_and(|c| c == "width_positive" || c == "height_positive") =>
            {
                Err(AdminError::InvalidDimensions.into())
            }
            sqlx::Error::Database(db_err)
                if db_err
                    .downcast_ref::<PgDatabaseError>()
                    .constraint()
                    .is_some_and(|c| c == "challenge_pkey") =>
            {
                Err(AdminError::NotUnique {
                    what: "Challenge url".into(),
                }
                .into())
            }
            other => Err(anyhow::Error::new(other)
                .context("failed to add challenge to database")
                .into()),
        };
    };
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteChallenge {
    pub url: String,
}

#[instrument(skip(state))]
pub async fn remove_challenge(
    State(state): State<Arc<AppState>>,
    Json(challenge): Json<DeleteChallenge>,
) -> super::Result<()> {
    let rows_affected = db::delete_challenge(&state.pool, &challenge.url).await?;
    match rows_affected {
        0 => Err(AdminError::NotFound(challenge.url).into()),
        _ => Ok(()),
    }
}

#[instrument(skip_all)]
pub async fn require_auth_mw(
    State(_state): State<Arc<AppState>>,
    _auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    request: Request,
    next: Next,
) -> Response {
    next.run(request).await
}
