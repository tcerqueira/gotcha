use std::sync::LazyLock;

use gotcha_server::test_helpers::{self, TestServer};
use reqwest::Client;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[tokio::test]
async fn site_verify() -> anyhow::Result<()> {
    let TestServer { port, .. } = test_helpers::create_server().await;

    let _response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/siteverify"))
        .form(&[("test", "this")])
        .send()
        .await?;

    Ok(())
}

#[tokio::test]
async fn challenge() -> anyhow::Result<()> {
    let TestServer { port, .. } = test_helpers::create_server().await;

    let _response = reqwest::get(format!("http://localhost:{port}/api/challenge")).await?;

    Ok(())
}

#[tokio::test]
async fn process_challenge() -> anyhow::Result<()> {
    let TestServer { port, .. } = test_helpers::create_server().await;

    let _response = HTTP_CLIENT
        .post(format!("http://localhost:{port}/api/process-challenge"))
        .send()
        .await?;

    Ok(())
}
