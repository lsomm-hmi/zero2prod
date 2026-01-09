use axum::body::{Body, HttpBody};
use axum::http::{Request, StatusCode};
use tower::ServiceExt; //for `oneshot`
use zero2prod::app;

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

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let addr = spawn_app().await;

    // Generate Http client
    let client = reqwest::Client::new();

    // Fetch response
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{addr}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(StatusCode::OK, response.status());
}

#[tokio::test]
async fn subscribe_returns_400_when_data_missing() {
    let addr = spawn_app().await;

    // Generate Http client
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Fetch response
        let response = client
            .post(&format!("{addr}/subscriptions"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            "The API did not fail with 400 Bad Request when the payload was {error_message}"
        );
    }
}
