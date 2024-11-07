use std::sync::LazyLock;

use gotcha_server::{
    routes::challenge::{ChallengeResponse, ChallengeResults, Claims, GetChallenge},
    test_helpers::{self, DEMO_API_SECRET_B64, DEMO_API_SECRET_B64URL, DEMO_JWT_SECRET_KEY_B64},
};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use reqwest::{Client, StatusCode};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[tokio::test]
async fn get_challenge() -> anyhow::Result<()> {
    let server = test_helpers::create_server().await;
    let port = server.port();

    let response = reqwest::get(format!(
        "http://localhost:{port}/api/challenge?secret={DEMO_API_SECRET_B64URL}"
    ))
    .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let challenge: GetChallenge = response.json().await?;
    assert!(challenge
        .url
        .contains(&format!("secret={DEMO_API_SECRET_B64URL}")));

    Ok(())
}

#[tokio::test]
async fn process_successful_challenge() -> anyhow::Result<()> {
    let server = test_helpers::create_server().await;
    let port = server.port();

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/challenge/process"))
        .json(&ChallengeResults {
            success: true,
            secret: DEMO_API_SECRET_B64.into(),
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let ChallengeResponse { token } = response.json().await?;
    let token_data: TokenData<Claims> = jsonwebtoken::decode(
        &token,
        &DecodingKey::from_base64_secret(DEMO_JWT_SECRET_KEY_B64)?,
        &Validation::new(Algorithm::HS256),
    )?;
    assert_eq!(token_data.header.alg, Algorithm::HS256);
    assert!(token_data.claims.custom.success);

    Ok(())
}

#[tokio::test]
async fn process_failed_challenge() -> anyhow::Result<()> {
    let server = test_helpers::create_server().await;
    let port = server.port();

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/challenge/process"))
        .json(&ChallengeResults {
            success: false,
            secret: DEMO_API_SECRET_B64.into(),
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let ChallengeResponse { token } = response.json().await?;
    let token_data = jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_base64_secret(DEMO_JWT_SECRET_KEY_B64)?,
        &Validation::new(Algorithm::HS256),
    )?;
    assert_eq!(token_data.header.alg, Algorithm::HS256);
    assert!(!token_data.claims.custom.success);

    Ok(())
}

#[tokio::test]
async fn process_challenge_with_invalid_secret() -> anyhow::Result<()> {
    let server = test_helpers::create_server().await;
    let port = server.port();

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/challenge/process"))
        .json(&ChallengeResults {
            success: false,
            secret: "bXktd3Jvbmctc2VjcmV0".into(), // `my-wrong-secret` in base64
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    Ok(())
}
