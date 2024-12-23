use std::{future::Future, net::SocketAddr, sync::Arc};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::{oneshot::Sender, OnceCell};

use crate::{
    app, configuration,
    crypto::{self, KEY_SIZE},
    db::{self, DbChallenge},
    get_configuration, HTTP_CLIENT,
};

const DEMO_CONSOLE_LABEL_PREFIX: &str = "console_for_integration_tests";
const DEMO_USER: &str = "Bk9vgyK6FiQ0oMHDT3b4EfQoIVRDs3ZM@clients";

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
    std::env::set_var("SERVER_DIR", "../");
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
            application: mut app_conf,
            database: db_conf,
            ..
        } = get_configuration().context("failed to load configuration")?;
        crate::init_tracing();

        let addr = format!("{}:0", app_conf.host);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let addr = listener.local_addr()?;
        app_conf.port = addr.port();
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

    pub fn test_id(&self) -> &uuid::Uuid {
        &self.inner.test_id
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

    pub async fn db_api_site_key(&self) -> String {
        db::fetch_api_keys(&self.inner.pool, &self.db_console().await)
            .await
            .unwrap()
            .swap_remove(0)
            .site_key
    }

    pub async fn db_api_secret(&self) -> String {
        db::fetch_api_keys(&self.inner.pool, &self.db_console().await)
            .await
            .unwrap()
            .swap_remove(0)
            .secret
    }

    pub async fn db_enconding_key(&self) -> String {
        db::fetch_encoding_key_by_site_key(&self.inner.pool, &self.db_api_site_key().await)
            .await
            .unwrap()
            .expect("expected a encoding key to be created on setup")
    }

    pub async fn db_challenges(&self) -> Vec<DbChallenge> {
        db::fetch_challenges(&self.inner.pool).await.unwrap()
    }
}

async fn populate_demo(pool: &PgPool, test_id: &uuid::Uuid) -> sqlx::Result<()> {
    let mut txn = pool.begin().await?;

    let console_id = db::insert_console(
        &mut *txn,
        &format!("{DEMO_CONSOLE_LABEL_PREFIX}-{test_id}"),
        DEMO_USER,
    )
    .await?;
    db::insert_api_key(
        &mut *txn,
        &crypto::gen_base64_key::<KEY_SIZE>(),
        &console_id,
        &crypto::gen_base64_key::<KEY_SIZE>(),
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
    db::delete_challenge_like(&mut *txn, &format!("%{test_id}")).await?;
    txn.commit().await?;
    Ok(())
}

pub async fn auth_jwt() -> &'static str {
    #[derive(Debug, Serialize)]
    struct TokenRequest {
        client_id: &'static str,
        client_secret: &'static str,
        audience: &'static str,
        grant_type: &'static str,
    }

    #[derive(Debug, Deserialize)]
    struct TokenResponse {
        access_token: String,
        #[expect(dead_code)]
        token_type: String,
    }

    static AUTH_JWT: OnceCell<String> = OnceCell::const_new();
    AUTH_JWT
        .get_or_init(|| async {
            HTTP_CLIENT
                .post("https://dev-650a4wh1mgk55eiy.us.auth0.com/oauth/token")
                .json(&TokenRequest {
                    client_id: std::env::var("TEST_AUTH_CLIENT_ID")
                        .expect("env var TEST_AUTH_CLIENT_ID")
                        .leak(),
                    client_secret: std::env::var("TEST_AUTH_CLIENT_SECRET")
                        .expect("env var TEST_AUTH_CLIENT_SECRET")
                        .leak(),
                    audience: "https://console-rust-backend",
                    grant_type: "client_credentials",
                })
                .send()
                .await
                .unwrap()
                .json::<TokenResponse>()
                .await
                .unwrap()
                .access_token
        })
        .await
}
