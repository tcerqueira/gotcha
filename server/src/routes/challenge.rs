use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::extract::ConnectInfo;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::{Host, Url};

use super::errors::ChallengeError;
use crate::analysis::interaction::{Interaction, Score};
use crate::extractors::ThisOrigin;
use crate::{analysis, db, response_token, AppState};
use crate::{db::DbChallenge, response_token::ResponseClaims};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetChallenge {
    pub url: Url,
    pub width: u16,
    pub height: u16,
}

#[instrument(skip(state))]
pub async fn get_challenge(
    State(state): State<Arc<AppState>>,
    ThisOrigin(origin): ThisOrigin,
) -> Result<Json<GetChallenge>, ChallengeError> {
    let challenges = db::fetch_challenges(&state.pool)
        .await
        .context("failed to fetch challenges")?;
    let challenge = choose_challenge(challenges).unwrap_or_else(|| DbChallenge {
        url: format!("{origin}/im-not-a-robot/index.html"),
        width: 304,
        height: 78,
    });

    Ok(Json(challenge.try_into()?))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResults {
    pub success: bool,
    pub site_key: String,
    #[serde(with = "host_as_str")]
    pub hostname: Host,
    pub challenge: Url,
    pub interactions: Vec<Interaction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub token: String,
}

#[instrument(skip(state, results),
    fields(
        success = results.success,
        site_key = results.site_key,
        hostname = results.hostname.to_string(),
        challenge = results.challenge.to_string(),
    )
)]
pub async fn process_challenge(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(results): Json<ChallengeResults>,
) -> Result<Json<ChallengeResponse>, ChallengeError> {
    let Score(score) = analysis::interaction::interaction_analysis(&results.interactions);
    tracing::debug!("interaction analysis: Score({:?})", score);
    let score = match results.success {
        true => score,
        false => 0.,
    };

    Ok(Json(ChallengeResponse {
        token: response_token::encode(
            ResponseClaims { score, solver_addr: addr, hostname: results.hostname },
            &db::fetch_api_key_by_site_key(&state.pool, &results.site_key)
                .await
                .context("failed to fecth encoding key by api secret while processing challenge")?
                .ok_or(ChallengeError::InvalidSecret)?
                .encoding_key,
        )?,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreAnalysisRequest {
    pub site_key: String,
    #[serde(with = "host_as_str")]
    pub hostname: Host,
    pub interactions: Vec<Interaction>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "result")]
#[serde(rename_all = "kebab-case")]
pub enum PreAnalysisResponse {
    Success { response: ChallengeResponse },
    Failure,
}

#[instrument(skip(state, results),
    fields(
        site_key = results.site_key,
        hostname = results.hostname.to_string(),
    )
)]
pub async fn process_pre_analysis(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(results): Json<PreAnalysisRequest>,
) -> Result<Json<PreAnalysisResponse>, ChallengeError> {
    // TODO: look at cookies and other fingerprints
    let Score(score) = analysis::interaction::interaction_analysis(&results.interactions);
    tracing::debug!("interaction analysis: Score({:?})", score);

    let response = match score {
        0f32..0.5 => PreAnalysisResponse::Failure,
        0.5..=1. => PreAnalysisResponse::Success {
            response: ChallengeResponse {
                token: response_token::encode(
                    ResponseClaims { score, solver_addr: addr, hostname: results.hostname },
                    &db::fetch_api_key_by_site_key(&state.pool, &results.site_key)
                        .await
                        .context(
                            "failed to fecth encoding key by api secret while processing challenge",
                        )?
                        .ok_or(ChallengeError::InvalidSecret)?
                        .encoding_key,
                )?,
            },
        },
        _ => {
            return Err(ChallengeError::Unexpected(anyhow::anyhow!(
                "score not in range [0.0 .. 1.0]"
            )))
        }
    };

    Ok(Json(response))
}

fn choose_challenge(mut challenges: Vec<DbChallenge>) -> Option<DbChallenge> {
    match &challenges[..] {
        [] => None,
        // _ => challenges.swap_remove(rand::thread_rng().gen_range(0..challenges.len())),
        _ => Some(challenges.swap_remove(0)),
    }
}

impl TryFrom<DbChallenge> for GetChallenge {
    type Error = anyhow::Error;

    fn try_from(db_challenge: DbChallenge) -> Result<Self, Self::Error> {
        let url = Url::parse(&db_challenge.url)
            .with_context(|| format!("malformed challenge url: {}", db_challenge.url))?;

        Ok(GetChallenge {
            url,
            width: db_challenge.width as u16,
            height: db_challenge.height as u16,
        })
    }
}

mod host_as_str {
    use std::borrow::Cow;

    use serde::{Deserializer, Serializer};

    use super::*;

    pub fn serialize<S>(host: &Host, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&host.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Host, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = Cow::<'de, str>::deserialize(deserializer)?;
        Host::parse(&str).map_err(serde::de::Error::custom)
    }
}
