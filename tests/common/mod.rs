use zero2prod::startup::app;

pub async fn spawn_app() -> String {
    // Bind to a random free port
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Build the app
    let app = app();

    // Spawn the server
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server crashed");
    });

    format!("http://{addr}")
}
