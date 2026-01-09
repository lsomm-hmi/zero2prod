use axum::{
    Router, debug_handler,
    extract::{Form, Path},
    http::StatusCode,
    routing::{get, post},
};

#[derive(serde::Deserialize)]
struct SubscribeFormData {
    name: String,
    email: String,
}

#[debug_handler]
async fn greet(path: Option<Path<String>>) -> String {
    let name = path.map(|Path(n)| n).unwrap_or_else(|| "World".to_string());

    format!("Hello {}!", name)
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn subscribe(Form(sub_info): Form<SubscribeFormData>) -> StatusCode {
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
        .route("/subscriptions", post(subscribe))
        .route("/{name}", get(greet))
}
