use gotcha_server::{configuration::Config, db};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Config {
        application: app_conf,
        database: db_conf,
        ..
    } = gotcha_server::get_configuration()?;

    #[cfg(not(feature = "aws-lambda"))]
    gotcha_server::init_tracing();
    #[cfg(feature = "aws-lambda")]
    lambda_http::tracing::init_default_subscriber();

    let pool = db::connect_database(db_conf);
    // aka 'dev' profile
    #[cfg(debug_assertions)]
    let _ = gotcha_server::db_dev_populate(&pool).await;

    #[cfg(not(feature = "aws-lambda"))]
    {
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
    }

    #[cfg(feature = "aws-lambda")]
    {
        std::env::set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");
        let _ = lambda_http::run(gotcha_server::app(app_conf, pool)).await;
    }
    Ok(())
}
