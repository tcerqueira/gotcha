use gotcha_server::{configuration::Config, db};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(not(feature = "aws-lambda"))]
    hosted_main().await?;
    #[cfg(feature = "aws-lambda")]
    lambda_main().await?;

    Ok(())
}

#[cfg(not(feature = "aws-lambda"))]
async fn hosted_main() -> anyhow::Result<()> {
    gotcha_server::init_tracing();

    let Config { application: app_conf, database: db_conf, .. } =
        gotcha_server::get_configuration()?;
    tracing::info!(config = ?app_conf, "Application config");
    tracing::info!(config = ?db_conf, "Database config");

    let pool = db::connect_database(db_conf);
    _ = gotcha_server::db_dev_populate(&pool).await;

    let addr = format!("{}:{}", app_conf.host, app_conf.port);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(
        listener,
        gotcha_server::app(app_conf, pool)
            .into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();

    Ok(())
}

#[cfg(feature = "aws-lambda")]
async fn lambda_main() -> anyhow::Result<()> {
    lambda_http::tracing::init_default_subscriber();

    let Config { application: app_conf, database: db_conf, .. } =
        gotcha_server::get_configuration()?;
    tracing::info!(?app_conf, "Application config");
    tracing::info!(?db_conf, "Database config");

    let pool = db::connect_database(db_conf);

    lambda_http::run(gotcha_server::app(app_conf, pool))
        .await
        .unwrap();
    Ok(())
}
