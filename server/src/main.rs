use gotcha_server::{configuration::Config, db};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Config {
        application: app_conf,
        database: db_conf,
        ..
    } = gotcha_server::get_configuration()?;
    gotcha_server::init_tracing();

    let addr = format!("{}:{}", app_conf.host, app_conf.port);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let pool = db::connect_database(db_conf);
    sqlx::migrate!("../migrations").run(&pool).await?;

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
