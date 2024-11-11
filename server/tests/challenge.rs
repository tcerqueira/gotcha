use std::sync::LazyLock;

use gotcha_server::routes::challenge::{ChallengeResponse, ChallengeResults, Claims, GetChallenge};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use reqwest::{Client, StatusCode};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[gotcha_server_macros::integration_test]
async fn get_challenge(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let api_secret = server.db_api_secret().await;
    let api_secret_url = urlencoding::encode(&api_secret);

    let response = reqwest::get(format!(
        "http://localhost:{port}/api/challenge?secret={api_secret_url}"
    ))
    .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let challenge: GetChallenge = response.json().await?;
    assert!(challenge.url.contains(&format!("secret={api_secret_url}")));

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn process_successful_challenge(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let api_secret = server.db_api_secret().await;
    let enc_key = server.db_enconding_key().await;

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/challenge/process"))
        .json(&ChallengeResults {
            success: true,
            secret: api_secret,
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let ChallengeResponse { token } = response.json().await?;
    let token_data: TokenData<Claims> = jsonwebtoken::decode(
        &token,
        &DecodingKey::from_base64_secret(&enc_key)?,
        &Validation::new(Algorithm::HS256),
    )?;
    assert_eq!(token_data.header.alg, Algorithm::HS256);
    assert!(token_data.claims.custom.success);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn process_failed_challenge(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let api_secret = server.db_api_secret().await;
    let enc_key = server.db_enconding_key().await;

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/challenge/process"))
        .json(&ChallengeResults {
            success: false,
            secret: api_secret,
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let ChallengeResponse { token } = response.json().await?;
    let token_data = jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_base64_secret(&enc_key)?,
        &Validation::new(Algorithm::HS256),
    )?;
    assert_eq!(token_data.header.alg, Algorithm::HS256);
    assert!(!token_data.claims.custom.success);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn process_challenge_with_invalid_secret(server: TestContext) -> anyhow::Result<()> {
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
