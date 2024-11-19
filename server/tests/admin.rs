use std::sync::LazyLock;

use gotcha_server::{
    db,
    routes::admin::{AddChallenge, DeleteChallenge},
};
use reqwest::{Client, StatusCode};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[gotcha_server_macros::integration_test]
async fn add_challenge_successful(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let pool = server.pool();
    let nonce = uuid::Uuid::new_v4();
    let url = format!("https://integration-test.com/index.html?nonce={nonce}");

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
        .json(&AddChallenge {
            url: url.clone(),
            width: 50,
            height: 50,
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let challenges = db::fetch_challenges(pool).await?;
    assert!(challenges.iter().any(|c| c.url == url));

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn add_challenge_bad_url(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
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

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
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

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
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
    let nonce = uuid::Uuid::new_v4();
    let url = format!("https://integration-test.com/index.html?nonce={nonce}");

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
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
    let nonce = uuid::Uuid::new_v4();
    let url = format!("https://integration-test.com/index.html?nonce={nonce}");

    let response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/admin/challenge"))
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
        .json(&DeleteChallenge { url })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[gotcha_server_macros::integration_test]
async fn remove_challenge_not_found(server: TestContext) -> anyhow::Result<()> {
    let port = server.port();
    let nonce = uuid::Uuid::new_v4();
    let url = format!("https://integration-test.com/index.html?nonce={nonce}");

    let response = HTTP_CLIENT
        .delete(format!("http://localhost:{port}/api/admin/challenge"))
        .json(&DeleteChallenge { url })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
}
