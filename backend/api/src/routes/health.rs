use axum::{Extension, http::StatusCode};

use super::wiki::WikiBackend;

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "health",
    responses((status = 200, body = String))
)]
pub async fn health() -> &'static str {
    "ok"
}

#[utoipa::path(
    get,
    path = "/api/v1/health/ready",
    tag = "health",
    responses((status = 200, body = String), (status = 503, body = String))
)]
pub async fn readiness(
    Extension(backend): Extension<WikiBackend>,
) -> Result<&'static str, (StatusCode, &'static str)> {
    backend.readiness_check().await.map_err(|err| {
        tracing::warn!(error = %err, "wiki readiness check failed");
        (StatusCode::SERVICE_UNAVAILABLE, "not ready")
    })?;
    Ok("ready")
}
