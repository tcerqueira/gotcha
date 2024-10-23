use gotcha_server::test_helpers::{self, TestServer};

// static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[tokio::test]
async fn get_challenge() -> anyhow::Result<()> {
    let TestServer { port, .. } = test_helpers::create_server().await;

    let _response = reqwest::get(format!(
        "http://localhost:{port}/api/challenge?token=test_site_key"
    ))
    .await?;

    Ok(())
}
