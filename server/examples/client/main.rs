use axum::{
    BoxError, Form, Json, Router,
    extract::ConnectInfo,
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri, uri::Authority},
    response::Redirect,
    routing::post,
};
use axum_extra::extract::Host;
use axum_server::tls_rustls::RustlsConfig;
use gotcha_server::routes::verification::VerificationResponse;
use reqwest::Client;
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::LazyLock};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{Level, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}

#[tokio::main]
async fn main() {
    init_tracing();

    let config_tls = RustlsConfig::from_pem_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("self_signed_certs")
            .join("cert.pem"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("self_signed_certs")
            .join("key.pem"),
    )
    .await
    .unwrap();
    let ports = Ports { http: 8000, https: 8001 };

    tokio::spawn(redirect_http_to_https(ports));

    let addr = SocketAddr::from(([0, 0, 0, 0], ports.https));
    // let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // tracing::info!("listening on {}", listener.local_addr().unwrap());
    tracing::info!("listening on {}", addr);
    // axum::serve(
    //     listener,
    //     app().into_make_service_with_connect_info::<SocketAddr>(),
    // )
    // .await
    // .unwrap();
    axum_server::bind_rustls(addr, config_tls)
        .serve(app().into_make_service_with_connect_info::<SocketAddr>())
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

#[instrument(err(Debug, level = Level::ERROR))]
async fn submit(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Form(data): Form<HashMap<String, String>>,
) -> Result<(StatusCode, Json<VerificationResponse>), StatusCode> {
    let token = data
        .get("gotcha-response")
        .ok_or(StatusCode::FORBIDDEN)?
        .as_str();

    let verification: VerificationResponse = HTTP_CLIENT
        .post("http://localhost:8080/api/siteverify")
        .form(&[
            (
                "secret",
                "4BdwFU84HLqceCQbE90-U5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q",
            ),
            ("response", token),
            ("remoteip", &addr.ip().to_string()),
        ])
        .header("User-Agent", "")
        .send()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        .json()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    match verification.success {
        true => {
            tracing::info!("site verification successful");
            Ok((StatusCode::OK, Json(verification)))
        }
        false => {
            tracing::error!("site verification failed");
            Err(StatusCode::FORBIDDEN)
        }
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

async fn redirect_http_to_https(ports: Ports) {
    fn make_https(host: &str, uri: Uri, https_port: u16) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let authority: Authority = host.parse()?;
        let bare_host = match authority.port() {
            Some(port_struct) => authority
                .as_str()
                .strip_suffix(port_struct.as_str())
                .unwrap()
                .strip_suffix(':')
                .unwrap(), // if authority.port() is Some(port) then we can be sure authority ends with :{port}
            None => authority.as_str(),
        };

        parts.authority = Some(format!("{bare_host}:{https_port}").parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(&host, uri, ports.https) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], ports.http));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, redirect.into_make_service())
        .await
        .unwrap();
}
