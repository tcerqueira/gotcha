use crate::{app, init_tracing};
use std::net::SocketAddr;
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
