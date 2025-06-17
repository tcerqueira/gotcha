use gotcha_server::{
    HTTP_CLIENT,
    crypto::{Base64UrlSafe, KEY_SIZE},
    db::{self, RowsAffected},
    routes::console::{
        ApiKeyResponse, ConsoleResponse, CreateConsoleRequest, UpdateApiKeyRequest,
        UpdateConsoleRequest,
    },
    test_helpers,
};
use gotcha_server_macros::integration_test;
use rand::distr::{Alphanumeric, SampleString};
use reqwest::StatusCode;
use uuid::Uuid;

async fn post_console(port: u16) -> anyhow::Result<ConsoleResponse> {
    let label = Alphanumeric.sample_string(&mut rand::rng(), 7);
    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/console"))
        .bearer_auth(test_helpers::auth_jwt().await)
        .json(&CreateConsoleRequest { label: label.clone() })
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}

#[integration_test]
async fn get_consoles(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let console_id = server.db_console().await;

    let response = HTTP_CLIENT
        .get(format!("http://localhost:{port}/api/console"))
        .bearer_auth(test_helpers::auth_jwt().await)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let consoles: Vec<ConsoleResponse> = response.json().await?;
    assert!(consoles.iter().any(|c| c.id == console_id));

    Ok(())
}

#[integration_test]
async fn create_console(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let pool = server.pool();

    let label = Alphanumeric.sample_string(&mut rand::rng(), 7);
    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/console"))
        .bearer_auth(test_helpers::auth_jwt().await)
        .json(&CreateConsoleRequest { label: label.clone() })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let ConsoleResponse { id, .. } = response.json().await?;
    let db_id = db::fetch_console_by_label(pool, &label)
        .await?
        .unwrap_or_else(|| panic!("console '{label}' doesn't exist"));
    let RowsAffected(r_affected) = db::delete_console(pool, &id).await?;

    assert_eq!(db_id, id);
    assert!(r_affected > 0);

    Ok(())
}

#[integration_test]
async fn update_console(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let pool = server.pool();

    let ConsoleResponse { id, label } = post_console(port).await?;

    let response = HTTP_CLIENT
        .patch(format!("http://localhost:{port}/api/console/{id}"))
        .bearer_auth(test_helpers::auth_jwt().await)
        .json(&UpdateConsoleRequest { label: label.as_ref().map(|l| l[..6].into()) })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let RowsAffected(r_affected) = db::delete_console(pool, &id).await?;

    assert!(r_affected > 0);

    Ok(())
}

#[integration_test]
async fn update_nothing_console(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let pool = server.pool();

    let ConsoleResponse { id, label } = post_console(port).await?;
    let label = label.as_ref().unwrap();

    let response = HTTP_CLIENT
        .patch(format!("http://localhost:{port}/api/console/{id}"))
        .bearer_auth(test_helpers::auth_jwt().await)
        .json(&UpdateConsoleRequest { label: None })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let db_id = db::fetch_console_by_label(pool, label)
        .await?
        .unwrap_or_else(|| panic!("console '{label}' doesn't exist"));
    let RowsAffected(r_affected) = db::delete_console(pool, &id).await?;

    assert_eq!(db_id, id);
    assert!(r_affected > 0);

    Ok(())
}

#[integration_test]
async fn delete_console(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();

    let ConsoleResponse { id, .. } = post_console(port).await?;

    let response = HTTP_CLIENT
        .delete(format!("http://localhost:{port}/api/console/{id}"))
        .bearer_auth(test_helpers::auth_jwt().await)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[integration_test]
async fn delete_console_not_found(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let id = Uuid::new_v4();

    let response = HTTP_CLIENT
        .delete(format!("http://localhost:{port}/api/console/{id}"))
        .bearer_auth(test_helpers::auth_jwt().await)
        .send()
        .await?;
    // because the console id doesn't exist for the user, it short circuits in the MW
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    Ok(())
}

#[integration_test]
async fn get_api_keys(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let console_id = server.db_console().await;

    let response = HTTP_CLIENT
        .get(format!(
            "http://localhost:{port}/api/console/{console_id}/api-key"
        ))
        .bearer_auth(test_helpers::auth_jwt().await)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let _consoles: Vec<ApiKeyResponse> = response.json().await?;

    Ok(())
}

#[integration_test]
async fn gen_api_key(server: TestContext) -> anyhow::Result<()> {
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

    let ApiKeyResponse { site_key, .. } = response.json().await?;
    assert_eq!(site_key.as_str().len(), KEY_SIZE * 4 / 3);

    let db_res = db::fetch_api_keys(pool, &console_id).await?;
    assert!(db_res.iter().any(|k| k.site_key == site_key));

    Ok(())
}

#[integration_test]
async fn gen_api_key_configuration_not_found(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let console_id = Uuid::new_v4();

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

#[integration_test]
async fn gen_api_key_forbidden_console(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let pool = server.pool();

    let console_id = db::insert_console(pool, "another_console", "another_user").await?;

    let response = HTTP_CLIENT
        .post(format!(
            "http://localhost:{port}/api/console/{console_id}/api-key"
        ))
        .bearer_auth(test_helpers::auth_jwt().await)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    db::delete_console(pool, &console_id).await?;
    Ok(())
}

#[integration_test]
async fn update_api_key(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let console_id = server.db_console().await;
    let site_key = server.db_api_site_key().await;

    let response = HTTP_CLIENT
        .patch(format!(
            "http://localhost:{port}/api/console/{console_id}/api-key/{site_key}"
        ))
        .bearer_auth(test_helpers::auth_jwt().await)
        .json(&UpdateApiKeyRequest { label: Some("updated".into()) })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[integration_test]
async fn revoke_api_key(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let pool = server.pool();
    let console_id = server.db_console().await;

    let ApiKeyResponse { site_key, .. } = HTTP_CLIENT
        .post(format!(
            "http://localhost:{port}/api/console/{console_id}/api-key"
        ))
        .bearer_auth(test_helpers::auth_jwt().await)
        .send()
        .await?
        .json()
        .await?;

    let response = HTTP_CLIENT
        .delete(format!(
            "http://localhost:{port}/api/console/{console_id}/api-key/{site_key}"
        ))
        .bearer_auth(test_helpers::auth_jwt().await)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let db_keys = db::fetch_api_keys(pool, &console_id).await?;
    assert!(db_keys.iter().all(|k| k.site_key != site_key));

    Ok(())
}

#[integration_test]
async fn update_forbidden_api_key(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let pool = server.pool();
    let console_id = server.db_console().await;

    let (other_console_id, site_key) = create_api_key_on_another_console(port).await?;

    let response = HTTP_CLIENT
        .patch(format!(
            "http://localhost:{port}/api/console/{console_id}/api-key/{site_key}"
        ))
        .bearer_auth(test_helpers::auth_jwt().await)
        .json(&UpdateApiKeyRequest { label: Some("updated".into()) })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    db::delete_console(pool, &other_console_id).await?;

    Ok(())
}

#[integration_test]
async fn revoke_forbidden_api_key(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let pool = server.pool();
    let console_id = server.db_console().await;

    let (other_console_id, site_key) = create_api_key_on_another_console(port).await?;

    let response = HTTP_CLIENT
        .delete(format!(
            "http://localhost:{port}/api/console/{console_id}/api-key/{site_key}"
        ))
        .bearer_auth(test_helpers::auth_jwt().await)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    db::delete_console(pool, &other_console_id).await?;

    Ok(())
}

#[integration_test]
async fn add_origin(_server: TestContext) -> anyhow::Result<()> {
    Ok(())
}

#[integration_test]
async fn remove_origin(_server: TestContext) -> anyhow::Result<()> {
    Ok(())
}

async fn create_api_key_on_another_console(port: u16) -> anyhow::Result<(Uuid, Base64UrlSafe)> {
    // create console
    let label = Alphanumeric.sample_string(&mut rand::rng(), 7);
    let ConsoleResponse { id: console_id, .. } = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/console"))
        .bearer_auth(test_helpers::auth_jwt().await)
        .json(&CreateConsoleRequest { label })
        .send()
        .await?
        .json()
        .await?;
    // create api key
    let ApiKeyResponse { site_key, .. } = HTTP_CLIENT
        .post(format!(
            "http://localhost:{port}/api/console/{console_id}/api-key"
        ))
        .bearer_auth(test_helpers::auth_jwt().await)
        .send()
        .await?
        .json()
        .await?;

    Ok((console_id, site_key))
}
