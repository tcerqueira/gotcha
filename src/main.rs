use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    gotcha::init_tracing();
    let addr = SocketAddr::from(([127, 0, 0, 1], 8001));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, gotcha::app().layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}
