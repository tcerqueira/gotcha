use gotcha_server::{
    routes::challenge::{
        ChallengeResponse, ChallengeResults, GetChallenge, PowResponse, PreAnalysisResponse,
    },
    tokens::{
        response::{ResponseClaims, JWT_RESPONSE_ALGORITHM},
        Claims,
    },
    HTTP_CLIENT,
};
use jsonwebtoken::{DecodingKey, Validation};
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

// This test overtime gets more meaningless and untestable
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
    eprintln!("{token}");
    let token_data = jsonwebtoken::decode::<Claims<ResponseClaims>>(
        &token,
        &DecodingKey::from_base64_secret(&enc_key)?,
        &Validation::new(JWT_RESPONSE_ALGORITHM),
    )?;
    assert_eq!(token_data.header.alg, JWT_RESPONSE_ALGORITHM);
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
    eprintln!("{token}");
    let token_data = jsonwebtoken::decode::<Claims<ResponseClaims>>(
        &token,
        &DecodingKey::from_base64_secret(&enc_key)?,
        &Validation::new(JWT_RESPONSE_ALGORITHM),
    )?;
    assert_eq!(token_data.header.alg, JWT_RESPONSE_ALGORITHM);
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

// This test overtime gets more meaningless and untestable
#[gotcha_server_macros::integration_test]
async fn process_pre_analysis_success(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let site_key = server.db_api_site_key().await;

    let response = HTTP_CLIENT
        .post(format!(
            "http://localhost:{port}/api/challenge/process-pre-analysis"
        ))
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

    let _: PreAnalysisResponse = response.json().await?;

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn process_pre_analysis_failure(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let site_key = server.db_api_site_key().await;

    let response = HTTP_CLIENT
        .post(format!(
            "http://localhost:{port}/api/challenge/process-pre-analysis"
        ))
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

    let response: PreAnalysisResponse = response.json().await?;
    assert!(matches!(response, PreAnalysisResponse::Failure));

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn proof_of_work_challenge(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let site_key = server.db_api_site_key().await;

    let response = HTTP_CLIENT
        .get(format!(
            "http://localhost:{port}/api/challenge/proof-of-work?site_key={site_key}"
        ))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let response: PowResponse = response.json().await?;
    assert!(!response.token.is_empty());

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn proof_of_work_challenge_no_site_key(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();

    let response = HTTP_CLIENT
        .get(format!(
            "http://localhost:{port}/api/challenge/proof-of-work"
        ))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
}
