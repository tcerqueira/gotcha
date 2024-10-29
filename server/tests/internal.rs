use std::sync::LazyLock;

use gotcha_server::{
    response_token::JWT_SECRET_KEY_B64,
    routes::internal::{ChallengeResponse, ChallengeResults, Claims},
    test_helpers::{self, TestServer},
};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use reqwest::{Client, StatusCode};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[tokio::test]
async fn get_challenge() -> anyhow::Result<()> {
    let TestServer { port, .. } = test_helpers::create_server().await;

    let response = reqwest::get(format!(
        "http://localhost:{port}/api/challenge?token=test_site_key"
    ))
    .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn process_successful_challenge() -> anyhow::Result<()> {
    let TestServer { port, .. } = test_helpers::create_server().await;

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/process-challenge"))
        .json(&ChallengeResults { success: true })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let ChallengeResponse { token } = response.json().await?;
    let token_data: TokenData<Claims> = jsonwebtoken::decode(
        &token,
        &DecodingKey::from_base64_secret(JWT_SECRET_KEY_B64)?,
        &Validation::new(Algorithm::HS256),
    )?;
    assert_eq!(token_data.header.alg, Algorithm::HS256);
    assert!(token_data.claims.custom.success);

    Ok(())
}

#[tokio::test]
async fn process_failed_challenge() -> anyhow::Result<()> {
    let TestServer { port, .. } = test_helpers::create_server().await;

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/process-challenge"))
        .json(&ChallengeResults { success: false })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let ChallengeResponse { token } = response.json().await?;
    let token_data = jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_base64_secret(JWT_SECRET_KEY_B64)?,
        &Validation::new(Algorithm::HS256),
    )?;
    assert_eq!(token_data.header.alg, Algorithm::HS256);
    assert!(!token_data.claims.custom.success);

    Ok(())
}
