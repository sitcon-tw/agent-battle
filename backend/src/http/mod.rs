mod bootstrap;
mod health;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(health::router())
        .merge(bootstrap::router())
}
