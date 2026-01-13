mod common;

use axum::body::{Body, HttpBody};
use axum::http::{Request, StatusCode};
use tower::ServiceExt; //for `oneshot`
use zero2prod::startup::app;

// In-process test using Axum + Tower. Faster, but doesn't utilize TCP/HTTP
#[tokio::test]
async fn health_check_works() {
    let app_state = common::make_app_state().await;
    let app = app(app_state);

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
    let test_app = common::spawn_app().await;

    // Generate Http client
    let client = reqwest::Client::new();

    // Fetch response (reqwest)
    let response = client
        .get(format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    //Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
