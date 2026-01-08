use axum::body::{Body, HttpBody};
use axum::http::{Request, StatusCode};
use tower::ServiceExt; //for `oneshot`
use zero2prod::app;

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
