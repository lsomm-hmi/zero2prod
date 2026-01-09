use crate::routes::*;
use axum::{
    Router,
    routing::{get, post},
};

pub async fn run() {
    let app = app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let _ = axum::serve(listener, app).await;
}

pub fn app() -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
}
