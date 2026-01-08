use axum::body::{Body, HttpBody};
use axum::http::{Request, StatusCode};
use tower::ServiceExt; //for `oneshot`
use zero2prod::app;

// In-process test using Axum + Tower. Faster, but doesn't utilize TCP/HTTP
#[tokio::test]
async fn health_check_works() {
    let app = app();

    // `Router` implements `tower::Service<Request<Body>>` so we can
    // call it like any tower service, no need to run an HTTP server.
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health_check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(Some(0), response.body().size_hint().exact());
}

// Integration Testing which generates an Http server to simulate external clients
#[tokio::test]
async fn health_check_integration_test() {
    let addr = spawn_app().await;

    // Generate Http client
    let client = reqwest::Client::new();

    // Fetch response (reqwest)
    let response = client
        .get(format!("{addr}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    //Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn spawn_app() -> String {
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
