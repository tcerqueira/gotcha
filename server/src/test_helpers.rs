use std::net::SocketAddr;

use crate::{app, configuration, db, get_configuration};
use tokio::task::JoinHandle;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub static DEMO_JWT_SECRET_KEY_B64: &str =
    "dHsFxb7mDHNv+cuI1L9GDW8AhXdWzuq/pwKWceDGq1SG4y2WD7zBwtiY2LHWNg3m";
pub static DEMO_API_SECRET_B64: &str =
    "4BdwFU84HLqceCQbE90+U5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q";
pub static DEMO_API_SECRET_B64URL: &str =
    "4BdwFU84HLqceCQbE90%2BU5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q";

pub struct TestServer {
    port: u16,
    join_handle: JoinHandle<()>,
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

impl TestServer {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn join_handle(&self) -> &JoinHandle<()> {
        &self.join_handle
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        tokio::runtime::Handle::current().spawn(async move {
            tracing::info!("server destroyed");
        });
    }
}
