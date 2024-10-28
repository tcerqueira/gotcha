use std::{sync::LazyLock, time::Duration};

use gotcha_server::{
    routes::internal::{
        ChallengeResponse, ChallengeResults, Claims, TMP_SECRET_KEY, TOKEN_TIMEOUT_SECS,
    },
    test_helpers::{self, TestServer},
};
use jsonwebtoken::{errors::ErrorKind, Algorithm, DecodingKey, TokenData, Validation};
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
        &DecodingKey::from_base64_secret(TMP_SECRET_KEY)?,
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
        &DecodingKey::from_base64_secret(TMP_SECRET_KEY)?,
        &Validation::new(Algorithm::HS256),
    )?;
    assert_eq!(token_data.header.alg, Algorithm::HS256);
    assert!(!token_data.claims.custom.success);

    Ok(())
}

#[ignore = "takes 30 secs waiting for token to expire"]
#[tokio::test]
async fn process_timedout_challenge() -> anyhow::Result<()> {
    let TestServer { port, .. } = test_helpers::create_server().await;

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/process-challenge"))
        .json(&ChallengeResults { success: false })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    tokio::time::sleep(Duration::from_secs(TOKEN_TIMEOUT_SECS + 1)).await;
    let ChallengeResponse { token } = response.json().await?;

    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0;
    let token_data = jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_base64_secret(TMP_SECRET_KEY)?,
        &validation,
    );

    let Err(err) = token_data else {
        anyhow::bail!("should be invalid due to timeout");
    };
    assert_eq!(err.into_kind(), ErrorKind::ExpiredSignature);

    Ok(())
}
