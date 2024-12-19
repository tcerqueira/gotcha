use gotcha_server::{
    routes::admin::{AddChallenge, DeleteChallenge},
    test_helpers, HTTP_CLIENT,
};
use reqwest::StatusCode;

#[gotcha_server_macros::integration_test]
async fn add_challenge_successful(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let auth_key = test_helpers::auth_jwt().await;
    let nonce = server.test_id();
    let url = format!("https://integration-test.com/index.html?nonce={nonce}");

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
        .header("Authorization", format!("Bearer {auth_key}"))
        .json(&AddChallenge {
            url: url.clone(),
            width: 50,
            height: 50,
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let challenges = server.db_challenges().await;
    assert!(challenges.iter().any(|c| c.url == url));

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn add_challenge_bad_url(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let auth_key = test_helpers::auth_jwt().await;

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
        .header("Authorization", format!("Bearer {auth_key}"))
        .json(&AddChallenge {
            url: "bad_url::integration-test.com/index.html".into(),
            width: 50,
            height: 50,
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn add_challenge_negative_dimensions(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let auth_key = test_helpers::auth_jwt().await;

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
        .header("Authorization", format!("Bearer {auth_key}"))
        .json(&serde_json::json!({
            "url": "https://integration-test.com/index.html",
            "width": -1,
            "height": 50
        }))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn add_challenge_zero_dimensions(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let auth_key = test_helpers::auth_jwt().await;

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
        .header("Authorization", format!("Bearer {auth_key}"))
        .json(&AddChallenge {
            url: "https://integration-test.com/index.html".into(),
            width: 50,
            height: 0,
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn add_challenge_already_exists(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let auth_key = test_helpers::auth_jwt().await;
    let nonce = server.test_id();
    let url = format!("https://integration-test.com/index.html?nonce={nonce}");

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
        .header("Authorization", format!("Bearer {auth_key}"))
        .json(&AddChallenge {
            url: url.clone(),
            width: 50,
            height: 50,
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
        .header("Authorization", format!("Bearer {auth_key}"))
        .json(&AddChallenge {
            url: url.clone(),
            width: 50,
            height: 50,
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::CONFLICT);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn remove_challenge_successful(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let auth_key = test_helpers::auth_jwt().await;
    let nonce = server.test_id();
    let url = format!("https://integration-test.com/index.html?nonce={nonce}");

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
        .header("Authorization", format!("Bearer {auth_key}"))
        .json(&AddChallenge {
            url: url.clone(),
            width: 50,
            height: 50,
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let response = HTTP_CLIENT
        .delete(format!("http://localhost:{port}/api/admin/challenge"))
        .header("Authorization", format!("Bearer {auth_key}"))
        .json(&DeleteChallenge { url })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn remove_challenge_not_found(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let auth_key = test_helpers::auth_jwt().await;
    let nonce = server.test_id();
    let url = format!("https://integration-test.com/index.html?nonce={nonce}");

    let response = HTTP_CLIENT
        .delete(format!("http://localhost:{port}/api/admin/challenge"))
        .header("Authorization", format!("Bearer {auth_key}"))
        .json(&DeleteChallenge { url })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn challenge_endpoint_missing_auth_key(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let nonce = server.test_id();

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
        .json(&AddChallenge {
            url: format!("https://integration-test.com/index.html?nonce={nonce}"),
            width: 50,
            height: 50,
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn challenge_endpoint_wrong_auth_key(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let nonce = server.test_id();

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
        .header("Authorization", "Bearer wrong-auth-key")
        .json(&AddChallenge {
            url: format!("https://integration-test.com/index.html?nonce={nonce}"),
            width: 50,
            height: 50,
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}
