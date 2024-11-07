use std::net::SocketAddr;

use crate::{app, configuration, db, get_configuration};
use sqlx::PgExecutor;
use tokio::sync::oneshot::Sender;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub static DEMO_JWT_SECRET_KEY_B64: &str =
    "dHsFxb7mDHNv+cuI1L9GDW8AhXdWzuq/pwKWceDGq1SG4y2WD7zBwtiY2LHWNg3m";
pub static DEMO_API_SECRET_B64: &str =
    "4BdwFU84HLqceCQbE90+U5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q";
pub static DEMO_API_SECRET_B64URL: &str =
    "4BdwFU84HLqceCQbE90%2BU5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q";

pub struct TestContext {
    port: u16,
    shutdown_signal: Option<Sender<()>>,
}

pub async fn create_test_context() -> TestContext {
    init_tracing();
    let configuration::Config {
        application: app_conf,
        database: db_conf,
        ..
    } = get_configuration().expect("failed to load configuration");

    let addr = format!("{}:0", app_conf.host);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let (shutdown_signal, shutdown_receiver) = tokio::sync::oneshot::channel();

    let pool = db::connect_database(db_conf);
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("failed to migrate db");
    populate_demo(&pool).await;

    tokio::spawn(async move {
        let app_pool = pool.clone();
        axum::serve(
            listener,
            app(app_conf, app_pool).into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move { shutdown_receiver.await.unwrap() })
        .await
        .unwrap();

        rollback_demo(&pool).await;
    });

    TestContext {
        port,
        shutdown_signal: Some(shutdown_signal),
    }
}

async fn populate_demo(_exec: impl PgExecutor<'_> + Send) {}
async fn rollback_demo(_exec: impl PgExecutor<'_> + Send) {}

pub fn init_tracing() {
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer().with_test_writer())
        .try_init();
}

impl TestContext {
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        if let Some(shutdown_signal) = self.shutdown_signal.take() {
            let _ = shutdown_signal.send(());
        }
    }
}
