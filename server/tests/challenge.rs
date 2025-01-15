use gotcha_server::{
    response_token::Claims,
    routes::challenge::{ChallengeResponse, ChallengeResults, GetChallenge},
    HTTP_CLIENT,
};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use reqwest::StatusCode;
use url::{Host, Url};

#[gotcha_server_macros::integration_test]
async fn get_challenge(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();

    let response = reqwest::get(format!("http://localhost:{port}/api/challenge")).await?;
    assert_eq!(response.status(), StatusCode::OK);
    let _challenge: GetChallenge = response.json().await?;

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn process_successful_challenge(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let site_key = server.db_api_site_key().await;
    let enc_key = server.db_enconding_key().await;

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/challenge/process"))
        .json(&ChallengeResults {
            success: true,
            site_key,
            hostname: Host::parse("website-integration.test.com")?,
            challenge: Url::parse("https://gotcha-integration.test.com/im-not-a-robot/index.html")?,
            interactions: vec![],
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
    // assert!(token_data.claims.custom.score >= 0.5);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn process_failed_challenge(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let site_key = server.db_api_site_key().await;
    let enc_key = server.db_enconding_key().await;

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/challenge/process"))
        .json(&ChallengeResults {
            success: false,
            site_key,
            hostname: Host::parse("website-integration.test.com")?,
            challenge: Url::parse("https://gotcha-integration.test.com/im-not-a-robot/index.html")?,
            interactions: vec![],
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
    assert!(token_data.claims.custom.score == 0.);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn process_challenge_with_invalid_secret(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/challenge/process"))
        .json(&ChallengeResults {
            success: false,
            site_key: "bXktd3Jvbmctc2VjcmV0".into(), // `my-wrong-secret` in base64
            hostname: Host::parse("website-integration.test.com")?,
            challenge: Url::parse("https://gotcha-integration.test.com/im-not-a-robot/index.html")?,
            interactions: vec![],
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    Ok(())
}
