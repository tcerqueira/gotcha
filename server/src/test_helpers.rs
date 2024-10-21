use crate::app;
use std::{net::SocketAddr, sync::OnceLock};
use tokio::task::JoinHandle;

pub struct TestServer {
    pub port: u16,
    pub join_handle: JoinHandle<()>,
}

pub async fn create_server() -> TestServer {
    init_tracing();

    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let join_handle = tokio::spawn(async move {
        axum::serve(listener, app()).await.unwrap();
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
