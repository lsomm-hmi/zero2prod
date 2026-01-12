use axum::{
    Router,
    routing::{get, post},
};
use tokio::net::TcpListener;

use crate::routes::{health_check, subscribe};
use crate::state::AppState;

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(state)
}

pub async fn run(listener: TcpListener, state: AppState) -> Result<(), std::io::Error> {
    axum::serve(listener, app(state)).await
}
