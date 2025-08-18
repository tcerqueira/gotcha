//! `/api/challenge` routes.

use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::{
    Json,
    extract::{ConnectInfo, Query, State},
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tracing::{Level, Span, instrument};
use url::{Host, Url};

use super::errors::ChallengeError;
use crate::{
    AppState,
    analysis::{
        self,
        interaction::{Interaction, Score},
        proof_of_work::PowChallenge,
    },
    db::{self, DbChallenge},
    encodings::{Base64, UrlSafe},
    tokens::{
        self, pow_challenge,
        response::{self, ResponseClaims},
    },
};

/// Expected params for get challenge route.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeParams {
    pub site_key: Option<Base64<UrlSafe>>,
}

/// Response payload of get challenge route.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetChallenge {
    /// Public URL.
    pub url: Url,
    /// Desktop width.
    pub width: u16,
    /// Desktop height.
    pub height: u16,
    /// Mobile width.
    pub small_width: u16,
    /// Mobile height.
    pub small_height: u16,
    /// Custom logo URL.
    pub logo_url: Option<String>,
}

/// Fetches challenges a responds with one of them randomly and its customization.
/// If `site_key` param is absent it responds with the defaults.
#[instrument(skip(state), err(Debug, level = Level::ERROR))]
pub async fn get_challenge(
    Query(query): Query<ChallengeParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<GetChallenge>, ChallengeError> {
    let challenges = match query.site_key {
        Some(site_key) => db::fetch_challenges_with_customization(&state.pool, &site_key).await,
        None => db::fetch_challenges(&state.pool).await,
    }
    .context("failed to fetch challenges")?;
    let challenge = choose_challenge(challenges).ok_or(ChallengeError::NoMatchingChallenge)?;

    Ok(Json(challenge.try_into()?))
}

/// Expected params for get proof of work route.
#[derive(Debug, Serialize, Deserialize)]
pub struct PowParams {
    /// Public site key encoded in base64 url safe alphabet.
    pub site_key: Base64<UrlSafe>,
}

/// Response payload of get proof of work route.
#[derive(Debug, Serialize, Deserialize)]
pub struct PowResponse {
    /// JWT with proof of work challenge.
    pub token: String,
}

/// Constructs a unique proof of work challenge and encodes it in a JWT.
/// Difficulty hardcoded to 3. Future work may include customizing it in an admin page.
#[instrument(skip(state), err(Debug, level = Level::ERROR))]
pub async fn get_proof_of_work_challenge(
    Query(query): Query<PowParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PowResponse>, ChallengeError> {
    let enc_key = db::fetch_api_key_by_site_key(&state.pool, &query.site_key)
        .await
        .context("failed to fetch api key by site key while getting proof of work")?
        .ok_or(ChallengeError::InvalidKey)?
        .encoding_key;

    Ok(Json(PowResponse {
        token: pow_challenge::encode(PowChallenge::random(3), &enc_key)
            .context("failed encoding jwt response")?,
    }))
}

/// Expected payload for processing challenge route.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResults {
    /// Wether or not successful.
    pub success: bool,
    /// Public site key encoded in base64 url safe alphabet.
    pub site_key: Base64<UrlSafe>,
    /// The host name of the URL where it was solved.
    #[serde(with = "crate::serde::host_as_str")]
    pub hostname: Host,
    /// The challenge URL that it was solved.
    pub challenge: Url,
    /// The list of interactions performed while solving the challenge.
    #[serde(default)]
    pub interactions: Vec<Interaction>,
}

/// Response payload of processing the challenge route.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResponse {
    /// JWT as proof of the challenge solution.
    pub token: String,
}

/// Proccesses the challenge results and responds with a proof in the form of a JWT.
#[instrument(skip(state, results), ret(Debug, level = Level::INFO), err(Debug, level = Level::ERROR),
    fields(
        ?addr,
        success = results.success,
        %site_key = results.site_key,
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
                .context("failed to fetch api key by site key while processing challenge")?
                .ok_or(ChallengeError::InvalidKey)?
                .encoding_key,
        )
        .context("failed encoding jwt response")?,
    }))
}

/// Expected payload for pre analysis route.
#[derive(Debug, Serialize, Deserialize)]
pub struct PreAnalysisRequest {
    /// Public site key encoded in base64 url safe alphabet.
    pub site_key: Base64<UrlSafe>,
    #[serde(with = "crate::serde::host_as_str")]
    /// The host name of the URL where it was solved.
    pub hostname: Host,
    /// The list of interactions performed while solving the challenge.
    pub interactions: Vec<Interaction>,
    /// Proof of work computed by the client.
    pub proof_of_work: ProofOfWork,
}

/// Proof of work containing the challenge in JWT and the solution to verify.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProofOfWork {
    /// JWT with proof of work challenge.
    pub challenge: String,
    /// Solution for the proof of work challenge.
    pub solution: u32,
}

impl ProofOfWork {
    pub fn verify(&self, dec_key: &Base64) -> Result<bool, jsonwebtoken::errors::Error> {
        let pow_challenge = tokens::pow_challenge::decode(&self.challenge, dec_key.as_str())
            .inspect_err(|_| {
                Span::current().record("pow_jwt", &self.challenge);
            })?;
        Span::current().record("pow_decoded", tracing::field::debug(&pow_challenge));

        Ok(pow_challenge.verify_solution(self.solution))
    }
}

/// Response payload of pre analysis route.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "result")]
#[serde(rename_all = "kebab-case")]
pub enum PreAnalysisResponse {
    /// Success case, the pre analysis is confident it's a trusted computer.
    Success { response: ChallengeResponse },
    /// Failure case, the user will then be required to solve a captcha.
    Failure,
}

/// The pre analysis is an ergonomic mechanism to allow trusted users to skip captcha challenges.
/// If the pre analysis is successful it instantly responds with the token, otherwise the widget will
/// prompt the user to solve a captcha challenge.
///
/// The pre analysis consists on analysing user input and checking the proof of work. At the moment,
/// it is configured to always fail thus forcing the user to solve a captcha every time.
/// TODO: check fingerprint.
#[instrument(skip(state, request), ret(Debug, level = Level::INFO), err(Debug, level = Level::ERROR),
    fields(
        %site_key = request.site_key,
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
        .context("failed to fetch api key by api secret while processing pre analysis")?
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

/// Expected payload for acessibility route.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessibilityRequest {
    /// Public site key encoded in base64 url safe alphabet.
    pub site_key: Base64<UrlSafe>,
    /// The host name of the URL where it was solved.
    #[serde(with = "crate::serde::host_as_str")]
    pub hostname: Host,
    /// Proof of work computed by the client.
    pub proof_of_work: ProofOfWork,
}

/// Alternative process for accessibility users. At the moment, just checks proof of work.
/// TODO: check fingerprint.
#[instrument(skip(state, request), ret(Debug, level = Level::INFO), err(Debug, level = Level::ERROR),
    fields(
        ?addr,
        ?site_key = request.site_key,
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
        .context("failed to fetch api key by api secret while processing accessility challenge")?
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
        _ => Some(challenges.swap_remove(rand::rng().random_range(0..challenges.len()))),
        // _ => Some(challenges.swap_remove(0)),
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
            small_width: db_challenge.small_width as u16,
            small_height: db_challenge.small_height as u16,
            logo_url: db_challenge.logo_url,
        })
    }
}
