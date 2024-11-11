use std::sync::LazyLock;

use gotcha_server::routes::console::ApiSecretRequest;
use reqwest::{Client, StatusCode};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[gotcha_server_macros::integration_test]
async fn gen_api_secret(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let console_id = server.db_console().await;

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/console/api-secret"))
        .json(&ApiSecretRequest {
            console_id,
            hostnames: vec![],
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn gen_api_secret_configuration_not_found(_server: TestContext) -> anyhow::Result<()> {
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
