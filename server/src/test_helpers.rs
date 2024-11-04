use std::net::SocketAddr;

use crate::{app, configuration, db, get_configuration};
use tokio::task::JoinHandle;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct TestServer {
    pub port: u16,
    pub join_handle: JoinHandle<()>,
}

pub async fn create_server() -> TestServer {
    init_tracing();
    let configuration::Config {
        application: app_conf,
        database: db_conf,
        ..
    } = get_configuration().expect("failed to load configuration");

    let addr = format!("{}:0", app_conf.host);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let join_handle = tokio::spawn(async move {
        let pool = db::connect_database(db_conf);
        sqlx::migrate!("../migrations")
            .run(&pool)
            .await
            .expect("failed to migrate db");

        axum::serve(
            listener,
            app(app_conf, pool).into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    TestServer { port, join_handle }
}

pub fn init_tracing() {
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer().with_test_writer())
        .try_init();
}
