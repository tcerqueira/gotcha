use gotcha_server::{
    crypto::KEY_SIZE,
    db,
    routes::console::{ApiSecret, ConsoleResponse, CreateConsoleRequest},
    test_helpers, HTTP_CLIENT,
};
use rand::distributions::{Alphanumeric, DistString};
use reqwest::StatusCode;

#[gotcha_server_macros::integration_test]
async fn create_console(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let pool = server.pool();

    let label = Alphanumeric.sample_string(&mut rand::thread_rng(), 7);
    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/console"))
        .bearer_auth(test_helpers::auth_jwt().await)
        .json(&CreateConsoleRequest {
            label: label.clone(),
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let ConsoleResponse { id, .. } = response.json().await?;
    let db_id = db::fetch_console_by_label(pool, &label)
        .await?
        .unwrap_or_else(|| panic!("console '{label}' doesn't exist"));
    let r_affected = db::delete_console(pool, &id).await?;

    assert_eq!(db_id, id);
    assert!(r_affected > 0);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn gen_api_secret(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let pool = server.pool();
    let console_id = server.db_console().await;

    let response = HTTP_CLIENT
        .post(format!(
            "http://localhost:{port}/api/console/{console_id}/api-key"
        ))
        .bearer_auth(test_helpers::auth_jwt().await)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let ApiSecret { site_key, .. } = response.json().await?;
    assert_eq!(site_key.len(), KEY_SIZE * 4 / 3);

    let db_res = db::fetch_api_keys(pool, &console_id).await?;
    assert!(db_res.iter().any(|k| k.site_key == site_key));

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn gen_api_secret_configuration_not_found(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let console_id = uuid::Uuid::new_v4();

    let response = HTTP_CLIENT
        .post(format!(
            "http://localhost:{port}/api/console/{console_id}/api-key"
        ))
        .bearer_auth(test_helpers::auth_jwt().await)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

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
