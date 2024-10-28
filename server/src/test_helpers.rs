use crate::{app, get_configuration};
use std::sync::OnceLock;
use tokio::task::JoinHandle;

pub struct TestServer {
    pub port: u16,
    pub join_handle: JoinHandle<()>,
}

pub async fn create_server() -> TestServer {
    // init_tracing();
    let config = get_configuration().expect("failed to load configuration");

    let addr = format!("{}:0", config.application.host);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let join_handle = tokio::spawn(async move {
        axum::serve(listener, app(config)).await.unwrap();
    });

    TestServer { port, join_handle }
}

pub fn init_tracing() {
    static TRACING: OnceLock<()> = OnceLock::new();
    TRACING.get_or_init(|| {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    });
}
