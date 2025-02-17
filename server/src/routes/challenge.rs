use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::extract::{ConnectInfo, Query};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::{instrument, Level};
use url::{Host, Url};

use super::errors::ChallengeError;
use super::extractors::ThisOrigin;
use crate::analysis::interaction::{Interaction, Score};
use crate::analysis::proof_of_work::PowChallenge;
use crate::tokens::{self, pow_challenge};
use crate::{
    analysis,
    db::{self, DbChallenge},
    tokens::response::{self, ResponseClaims},
    AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetChallenge {
    pub url: Url,
    pub width: u16,
    pub height: u16,
}

#[instrument(skip(state), err(Debug, level = Level::ERROR))]
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
pub struct PowSiteKey {
    pub site_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PowResponse {
    pub token: String,
}

#[instrument(skip(state), err(Debug, level = Level::ERROR))]
pub async fn get_proof_of_work_challenge(
    Query(query): Query<PowSiteKey>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PowResponse>, ChallengeError> {
    let enc_key = db::fetch_api_key_by_site_key(&state.pool, &query.site_key)
        .await
        .context("failed to fecth encoding key by api secret while processing challenge")?
        .ok_or(ChallengeError::InvalidKey)?
        .encoding_key;

    Ok(Json(PowResponse {
        token: pow_challenge::encode(PowChallenge::gen(4), &enc_key)
            .context("failed encoding jwt response")?,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResults {
    pub success: bool,
    pub site_key: String,
    #[serde(with = "crate::serde::host_as_str")]
    pub hostname: Host,
    pub challenge: Url,
    #[serde(default)]
    pub interactions: Vec<Interaction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub token: String,
}

#[instrument(skip(state, results), ret(Debug, level = Level::INFO), err(Debug, level = Level::ERROR),
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
    // TODO: potentially heavy CPU operation - offload to rayon
    let Score(score) = analysis::interaction::interaction_analysis(&results.interactions);
    tracing::debug!("interaction analysis: Score({:?})", score);
    let score = match results.success {
        true => {
            tracing::warn!("interaction analysis disabled");
            1.
        }
        false => 0.,
    };

    Ok(Json(ChallengeResponse {
        token: response::encode(
            ResponseClaims { score, addr: addr.ip(), host: results.hostname },
            &db::fetch_api_key_by_site_key(&state.pool, &results.site_key)
                .await
                .context("failed to fecth encoding key by api secret while processing challenge")?
                .ok_or(ChallengeError::InvalidKey)?
                .encoding_key,
        )
        .context("failed encoding jwt response")?,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreAnalysisRequest {
    pub site_key: String,
    #[serde(with = "crate::serde::host_as_str")]
    pub hostname: Host,
    pub interactions: Vec<Interaction>,
    pub proof_of_work: ProofOfWork,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofOfWork {
    pub challenge: String,
    pub solution: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "result")]
#[serde(rename_all = "kebab-case")]
pub enum PreAnalysisResponse {
    Success { response: ChallengeResponse },
    Failure,
}

#[instrument(skip(state, results), ret(Debug, level = Level::INFO), err(Debug, level = Level::ERROR),
    fields(
        site_key = results.site_key,
        hostname = results.hostname.to_string(),
        pow_jwt,
        pow_decoded,
        solution = results.proof_of_work.solution,
    )
)]
pub async fn process_pre_analysis(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(results): Json<PreAnalysisRequest>,
) -> Result<Json<PreAnalysisResponse>, ChallengeError> {
    // TODO: look at cookies and other fingerprints
    let dec_key = db::fetch_api_key_by_site_key(&state.pool, &results.site_key)
        .await
        .context("failed to fecth encoding key by api secret while processing challenge")?
        .ok_or(ChallengeError::InvalidKey)?
        .encoding_key;

    let pow_challenge = tokens::pow_challenge::decode(&results.proof_of_work.challenge, &dec_key)
        .inspect_err(|_| {
        tracing::Span::current().record("pow_jwt", &results.proof_of_work.challenge);
    })?;
    tracing::Span::current().record("pow_decoded", tracing::field::debug(&pow_challenge));

    let is_proof = pow_challenge.verify_solution(results.proof_of_work.solution);
    if !is_proof {
        return Err(ChallengeError::FailedProofOfWork);
    }

    // TODO: potentially heavy CPU operation - offload to rayon
    let Score(score) = analysis::interaction::interaction_analysis(&results.interactions);
    tracing::debug!("interaction analysis: Score({:?})", score);

    let response = match score {
        _ => PreAnalysisResponse::Failure,
        // For now, assume pre-analysis always fails
        #[expect(unreachable_patterns)]
        0f32..0.5 => PreAnalysisResponse::Failure,
        #[expect(unreachable_patterns)]
        0.5..=1. => PreAnalysisResponse::Success {
            response: ChallengeResponse {
                token: response::encode(
                    ResponseClaims { score, addr: addr.ip(), host: results.hostname },
                    &dec_key,
                )
                .context("failed encoding jwt response")?,
            },
        },
        #[expect(unreachable_patterns)]
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
