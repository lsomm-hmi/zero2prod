mod common;

use axum::http::StatusCode;

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let addr = common::spawn_app().await;

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
async fn subscribe_returns_422_when_data_missing() {
    let addr = common::spawn_app().await;

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
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status(),
            "The API did not fail with 400 Bad Request when the payload was {error_message}"
        );
    }
}
