use sqlx::PgPool;
use zero2prod::{configuration::get_configuration, startup::app, state::AppState};

pub async fn spawn_app() -> String {
    // Bind to a random free port
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Build the app
    let app_state = make_app_state().await;
    let app = app(app_state);

    // Spawn the server
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server crashed");
    });

    format!("http://{addr}")
}

pub async fn make_app_state() -> AppState {
    // Panic if we cannot read configuration
    let config = get_configuration().expect("Failed to read configuration.");

    let db = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let state = AppState { db, config };
    state
}
