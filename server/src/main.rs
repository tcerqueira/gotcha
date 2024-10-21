use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    gotcha_server::init_tracing();
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, gotcha_server::app()).await.unwrap();
}
