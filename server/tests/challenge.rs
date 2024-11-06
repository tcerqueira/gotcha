use std::sync::LazyLock;

use gotcha_server::{
    response_token::JWT_SECRET_KEY_B64,
    routes::challenge::{ChallengeResponse, ChallengeResults, Claims, GetChallenge},
    test_helpers,
};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use reqwest::{Client, StatusCode};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[tokio::test]
async fn get_challenge() -> anyhow::Result<()> {
    let server = test_helpers::create_server().await;
    let port = server.port();

    let response = reqwest::get(format!(
        "http://localhost:{port}/api/challenge?token=4BdwFU84HLqceCQbE90%2BU5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q"
    ))
    .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let challenge: GetChallenge = response.json().await?;
    assert!(challenge
        .url
        .contains("token=4BdwFU84HLqceCQbE90%2BU5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q"));

    Ok(())
}

#[tokio::test]
async fn process_successful_challenge() -> anyhow::Result<()> {
    let server = test_helpers::create_server().await;
    let port = server.port();

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
    let server = test_helpers::create_server().await;
    let port = server.port();

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
