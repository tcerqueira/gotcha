use gotcha_server::{routes::challenge::GetChallenge, test_helpers::DEMO_API_SECRET_B64URL};
use reqwest::StatusCode;

#[gotcha_server_macros::integration_test]
async fn gen_api_secret(server: TestContext) -> anyhow::Result<()> {
    server.port();
    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn gen_api_secret_configuration_not_found(server: TestContext) -> anyhow::Result<()> {
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

#[gotcha_server_macros::integration_test]
async fn add_origin(_server: TestContext) -> anyhow::Result<()> {
    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn remove_origin(_server: TestContext) -> anyhow::Result<()> {
    Ok(())
}
