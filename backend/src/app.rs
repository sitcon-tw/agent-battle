use axum::Router;
use tower_http::trace::TraceLayer;

use crate::http;

pub fn router() -> Router {
    Router::new()
        .merge(http::router())
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let response = router()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("valid request"),
            )
            .await
            .expect("router response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), 1024)
            .await
            .expect("body bytes");
        assert_eq!(&body[..], b"OK");
    }
}
