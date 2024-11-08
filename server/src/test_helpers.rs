use std::{future::Future, net::SocketAddr, sync::Arc};

use crate::{app, configuration, db, get_configuration};
use anyhow::Context;
use sqlx::{PgExecutor, PgPool};
use tokio::sync::oneshot::Sender;

pub static DEMO_JWT_SECRET_KEY_B64: &str =
    "dHsFxb7mDHNv+cuI1L9GDW8AhXdWzuq/pwKWceDGq1SG4y2WD7zBwtiY2LHWNg3m";
pub static DEMO_API_SECRET_B64: &str =
    "4BdwFU84HLqceCQbE90+U5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q";
pub static DEMO_API_SECRET_B64URL: &str =
    "4BdwFU84HLqceCQbE90%2BU5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q";

#[derive(Debug, Clone)]
pub struct TestContext {
    inner: Arc<InnerContext>,
}

#[derive(Debug)]
struct InnerContext {
    addr: SocketAddr,
    shutdown_signal: Sender<()>,
    pool: PgPool,
}

pub async fn with_test_context<F, Fut, R>(test: F) -> R
where
    F: FnOnce(TestContext) -> Fut,
    Fut: Future<Output = R>,
{
    let ctx = TestContext::setup()
        .await
        .expect("failed to setup test environment");
    let result = test(ctx.clone()).await;
    let _ = ctx.teardown().await;
    result
}

impl TestContext {
    pub async fn setup() -> anyhow::Result<Self> {
        crate::init_tracing();
        let configuration::Config {
            application: app_conf,
            database: db_conf,
            ..
        } = get_configuration().context("failed to load configuration")?;

        let addr = format!("{}:0", app_conf.host);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let addr = listener.local_addr()?;
        let (shutdown_signal, shutdown_receiver) = tokio::sync::oneshot::channel();

        let pool = db::connect_database(db_conf);
        sqlx::migrate!("../migrations")
            .run(&pool)
            .await
            .context("failed to migrate db")?;
        populate_demo(&pool).await;

        let app_pool = pool.clone();
        let _join_handle = tokio::spawn(async move {
            axum::serve(
                listener,
                app(app_conf, app_pool).into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(async move { shutdown_receiver.await.unwrap() })
            .await
            .unwrap();
        });

        Ok(Self {
            inner: Arc::new(InnerContext {
                addr,
                shutdown_signal,
                pool,
            }),
        })
    }

    pub async fn teardown(self) -> anyhow::Result<()> {
        let ctx = Arc::try_unwrap(self.inner)
            .expect("test context references should not leak after the test");
        rollback_demo(&ctx.pool).await;
        let _ = ctx.shutdown_signal.send(());
        tracing::info!("Shutting down server on {}", ctx.addr);
        Ok(())
    }

    pub fn port(&self) -> u16 {
        self.inner.addr.port()
    }

    pub fn pool(&self) -> &PgPool {
        &self.inner.pool
    }
}

async fn populate_demo(_exec: impl PgExecutor<'_> + Send) {}
async fn rollback_demo(_exec: impl PgExecutor<'_> + Send) {}
