//! Health check endpoint.

use axum::http::StatusCode;

/// Returns a successful health check response.
pub async fn health() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}
