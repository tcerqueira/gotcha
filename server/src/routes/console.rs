use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSecretRequest {
    pub console_id: uuid::Uuid,
    pub hostnames: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSecretResponse {
    secret: String,
}

pub async fn gen_api_secret(
    Json(_request): Json<ApiSecretRequest>,
) -> super::Result<Json<ApiSecretResponse>> {
    Ok(Json(ApiSecretResponse { secret: "".into() }))
}

pub async fn add_origin() -> StatusCode {
    todo!()
}

pub async fn remove_origin() -> StatusCode {
    todo!()
}
