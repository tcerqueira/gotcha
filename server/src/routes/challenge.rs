use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::{
    Json,
    extract::{ConnectInfo, Query, State},
};
use serde::{Deserialize, Serialize};
use tracing::{Level, Span, instrument};
use url::{Host, Url};

use super::{errors::ChallengeError, extractors::ThisOrigin};
use crate::{
    AppState,
    analysis::{
        self,
        interaction::{Interaction, Score},
        proof_of_work::PowChallenge,
    },
    db::{self, DbChallenge},
    tokens::{
        self, pow_challenge,
        response::{self, ResponseClaims},
    },
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
        .context("failed to fetch encoding key by api secret while processing challenge")?
        .ok_or(ChallengeError::InvalidKey)?
        .encoding_key;

    Ok(Json(PowResponse {
        token: pow_challenge::encode(PowChallenge::random(3), &enc_key)
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
        ?addr,
        success = results.success,
        site_key = results.site_key,
        ?hostname = results.hostname,
        %challenge = results.challenge,
        interaction_score,
    )
)]
pub async fn process_challenge(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(results): Json<ChallengeResults>,
) -> Result<Json<ChallengeResponse>, ChallengeError> {
    // TODO: potentially heavy CPU operation - offload to rayon
    let Score(score) = analysis::interaction::interaction_analysis(&results.interactions);
    Span::current().record("interaction_score", score);
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
                .context("failed to fetch encoding key by api secret while processing challenge")?
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

impl ProofOfWork {
    pub fn verify(&self, dec_key: &str) -> Result<bool, jsonwebtoken::errors::Error> {
        let pow_challenge =
            tokens::pow_challenge::decode(&self.challenge, dec_key).inspect_err(|_| {
                Span::current().record("pow_jwt", &self.challenge);
            })?;
        Span::current().record("pow_decoded", tracing::field::debug(&pow_challenge));

        Ok(pow_challenge.verify_solution(self.solution))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "result")]
#[serde(rename_all = "kebab-case")]
pub enum PreAnalysisResponse {
    Success { response: ChallengeResponse },
    Failure,
}

#[instrument(skip(state, request), ret(Debug, level = Level::INFO), err(Debug, level = Level::ERROR),
    fields(
        site_key = request.site_key,
        ?hostname = request.hostname,
        pow_jwt,
        pow_decoded,
        pow_solution = request.proof_of_work.solution,
        interaction_score,
    )
)]
pub async fn process_pre_analysis(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(request): Json<PreAnalysisRequest>,
) -> Result<Json<PreAnalysisResponse>, ChallengeError> {
    // TODO: look at cookies and other fingerprints
    let crypt_key = db::fetch_api_key_by_site_key(&state.pool, &request.site_key)
        .await
        .context("failed to fetch encoding key by api secret while processing challenge")?
        .ok_or(ChallengeError::InvalidKey)?
        .encoding_key;

    let verified = request.proof_of_work.verify(&crypt_key)?;
    if !verified {
        return Err(ChallengeError::FailedProofOfWork);
    }

    // TODO: potentially heavy CPU operation - offload to rayon
    let Score(score) = analysis::interaction::interaction_analysis(&request.interactions);
    Span::current().record("interaction_score", score);

    let response = match score {
        _ => PreAnalysisResponse::Failure,
        // For now, assume pre-analysis always fails
        #[expect(unreachable_patterns)]
        0f32..0.5 => PreAnalysisResponse::Failure,
        #[expect(unreachable_patterns)]
        0.5..=1. => PreAnalysisResponse::Success {
            response: ChallengeResponse {
                token: response::encode(
                    ResponseClaims { score, addr: addr.ip(), host: request.hostname },
                    &crypt_key,
                )
                .context("failed encoding jwt response")?,
            },
        },
        #[expect(unreachable_patterns)]
        _ => {
            return Err(ChallengeError::Unexpected(anyhow::anyhow!(
                "score not in range [0.0 .. 1.0]"
            )));
        }
    };

    Ok(Json(response))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessibilityRequest {
    pub site_key: String,
    #[serde(with = "crate::serde::host_as_str")]
    pub hostname: Host,
    pub proof_of_work: ProofOfWork,
}

#[instrument(skip(state, request), ret(Debug, level = Level::INFO), err(Debug, level = Level::ERROR),
    fields(
        ?addr,
        site_key = request.site_key,
        ?hostname = request.hostname,
        pow_jwt,
        pow_decoded,
        solution = request.proof_of_work.solution,
    )
)]
pub async fn process_accessibility_challenge(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(request): Json<AccessibilityRequest>,
) -> Result<Json<PreAnalysisResponse>, ChallengeError> {
    // TODO: look at cookies and other fingerprints
    let crypt_key = db::fetch_api_key_by_site_key(&state.pool, &request.site_key)
        .await
        .context("failed to fetch encoding key by api secret while processing challenge")?
        .ok_or(ChallengeError::InvalidKey)?
        .encoding_key;

    let verified = request.proof_of_work.verify(&crypt_key)?;
    if !verified {
        return Err(ChallengeError::FailedProofOfWork);
    }

    let token = response::encode(
        ResponseClaims { score: 1.0, addr: addr.ip(), host: request.hostname },
        &crypt_key,
    )?;

    Ok(Json(PreAnalysisResponse::Success {
        response: ChallengeResponse { token },
    }))
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
