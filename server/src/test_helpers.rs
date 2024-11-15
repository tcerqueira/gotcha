use std::{future::Future, net::SocketAddr, sync::Arc};

use anyhow::Context;
use sqlx::PgPool;
use tokio::sync::oneshot::Sender;

use crate::{
    app, configuration,
    crypto::{self, KEY_SIZE},
    db, get_configuration,
};

static DEMO_CONSOLE_LABEL_PREFIX: &str = "console_for_integration_tests";

#[derive(Debug, Clone)]
pub struct TestContext {
    inner: Arc<InnerContext>,
}

#[derive(Debug)]
struct InnerContext {
    test_id: uuid::Uuid,
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
        let configuration::Config {
            application: app_conf,
            database: db_conf,
            ..
        } = get_configuration().context("failed to load configuration")?;
        crate::init_tracing();

        let addr = format!("{}:0", app_conf.host);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let addr = listener.local_addr()?;
        let (shutdown_signal, shutdown_receiver) = tokio::sync::oneshot::channel();

        let pool = db::connect_database(db_conf);
        let test_id = uuid::Uuid::new_v4();
        populate_demo(&pool, &test_id).await?;

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
                test_id,
                addr,
                shutdown_signal,
                pool,
            }),
        })
    }

    pub async fn teardown(self) -> anyhow::Result<()> {
        let ctx = Arc::try_unwrap(self.inner)
            .expect("test context references should not leak after the test");
        let _ = rollback_demo(&ctx.pool, &ctx.test_id).await;
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

    pub async fn db_console(&self) -> uuid::Uuid {
        db::fetch_console_by_label(
            &self.inner.pool,
            &format!("{DEMO_CONSOLE_LABEL_PREFIX}-{}", self.inner.test_id),
        )
        .await
        .unwrap()
        .expect("expected a console to be created on setup")
    }

    pub async fn db_api_secret(&self) -> String {
        db::fetch_api_secrets(&self.inner.pool, &self.db_console().await)
            .await
            .unwrap()
            .swap_remove(0)
    }

    pub async fn db_enconding_key(&self) -> String {
        db::fetch_encoding_key(&self.inner.pool, &self.db_api_secret().await)
            .await
            .unwrap()
            .expect("expected a encoding key to be created on setup")
    }
}

async fn populate_demo(pool: &PgPool, test_id: &uuid::Uuid) -> sqlx::Result<()> {
    let mut txn = pool.begin().await?;

    let console_id =
        db::insert_console(&mut *txn, &format!("{DEMO_CONSOLE_LABEL_PREFIX}-{test_id}")).await?;
    db::insert_api_secret(
        &mut *txn,
        &crypto::gen_base64_key::<KEY_SIZE>(),
        &console_id,
        &crypto::gen_base64_key::<KEY_SIZE>(),
    )
    .await?;

    txn.commit().await?;
    Ok(())
}

async fn rollback_demo(pool: &PgPool, test_id: &uuid::Uuid) -> sqlx::Result<()> {
    let mut txn = pool.begin().await?;
    let id =
        db::fetch_console_by_label(&mut *txn, &format!("{DEMO_CONSOLE_LABEL_PREFIX}-{test_id}"))
            .await?
            .expect("expected a console to be created on setup");
    db::delete_console(&mut *txn, &id).await?;
    txn.commit().await?;
    Ok(())
}
