use axum::{extract::ConnectInfo, http::StatusCode, routing::post, Form, Json, Router};
use gotcha_server::routes::verification::VerificationResponse;
use reqwest::Client;
use std::{collections::HashMap, net::SocketAddr, sync::LazyLock};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    init_tracing();
    let addr = SocketAddr::from(([127, 0, 0, 1], 8001));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app().into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

fn app() -> Router {
    let serve_dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("server/examples/client")
        .join("assets");
    tracing::debug!("Serving files from: {:?}", serve_dir);

    Router::new()
        .route("/submit", post(submit))
        .fallback_service(ServeDir::new(serve_dir))
        .layer(TraceLayer::new_for_http())
}

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

async fn submit(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Form(data): Form<HashMap<String, String>>,
) -> Result<(StatusCode, Json<VerificationResponse>), StatusCode> {
    let token = data
        .get("g-recaptcha-response")
        .ok_or(StatusCode::FORBIDDEN)?
        .as_str();

    let verification: VerificationResponse = HTTP_CLIENT
        .post("http://localhost:8080/api/siteverify")
        .form(&[
            (
                "secret",
                "4BdwFU84HLqceCQbE90+U5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q",
            ),
            ("response", token),
            ("remoteip", &addr.ip().to_string()),
        ])
        .send()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        .json()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    match verification.success {
        true => Ok((StatusCode::OK, Json(verification))),
        false => Err(StatusCode::FORBIDDEN),
    }
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
