use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    gotcha_client::init_tracing();
    let addr = SocketAddr::from(([127, 0, 0, 1], 8001));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, gotcha_client::app()).await.unwrap();
}
