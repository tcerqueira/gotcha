use std::sync::LazyLock;

use gotcha_server::{
    crypto::KEY_SIZE,
    db,
    routes::console::{ApiSecretRequest, ApiSecretResponse, ConsoleRequest, ConsoleResponse},
};
use rand::distributions::{Alphanumeric, DistString};
use reqwest::{Client, StatusCode};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[gotcha_server_macros::integration_test]
async fn create_console(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let pool = server.pool();

    let label = Alphanumeric.sample_string(&mut rand::thread_rng(), 7);
    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/console"))
        .json(&ConsoleRequest {
            label: label.clone(),
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let ConsoleResponse { id } = response.json().await?;
    let db_id = db::fetch_console_by_label(pool, &label)
        .await?
        .unwrap_or_else(|| panic!("console with '{label}' doesn't exist"));
    assert_eq!(db_id, id);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn gen_api_secret(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let pool = server.pool();
    let console_id = server.db_console().await;

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/console/secret"))
        .json(&ApiSecretRequest { console_id })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let ApiSecretResponse { secret } = response.json().await?;
    assert_eq!(secret.len(), KEY_SIZE * 4 / 3);

    let db_res = db::fetch_api_secrets(pool, &console_id).await?;
    assert!(db_res.contains(&secret));

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
