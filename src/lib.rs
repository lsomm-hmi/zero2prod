use axum::{Router, debug_handler, extract::Path, http::StatusCode, routing::get};

#[debug_handler]
async fn greet(path: Option<Path<String>>) -> String {
    let name = path.map(|Path(n)| n).unwrap_or_else(|| "World".to_string());

    format!("Hello {}!", name)
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub async fn run() {
    let app = app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let _ = axum::serve(listener, app).await;
}

pub fn app() -> Router {
    Router::new()
        .route("/", get(greet))
        .route("/health_check", get(health_check))
        .route("/{name}", get(greet))
}
