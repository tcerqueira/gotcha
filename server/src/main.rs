use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gotcha_server::init_tracing();
    let config = gotcha_server::get_configuration()?;

    let addr = format!("{}:{}", config.application.host, config.application.port);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(
        listener,
        gotcha_server::app(config).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
    Ok(())
}
