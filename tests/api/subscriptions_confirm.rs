use crate::helpers::{ConfirmationLinks, spawn_app};
use axum::http::StatusCode;
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
pub async fn confirmations_without_token_are_rejected_with_400() {
    // Arrange
    let test_app = spawn_app().await;

    // Act
    let response = reqwest::get(&format!("{}/subscriptions/confirm", test_app.address))
        .await
        .unwrap();

    // Assert
    assert_eq!(StatusCode::BAD_REQUEST, response.status());
}

#[tokio::test]
pub async fn link_returned_by_subscribe_returns_200_if_called() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await;
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];

    // Extract link from request fields
    let ConfirmationLinks {
        html: confirmation_link,
        plain_text: _,
    } = test_app.get_confirmation_links(email_request);

    // Act
    let response = reqwest::get(confirmation_link).await.unwrap();

    // Assert
    assert_eq!(StatusCode::OK, response.status())
}

#[tokio::test]
async fn clicking_confirmation_link_confirms_subscriber() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await;
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let ConfirmationLinks {
        html: confirmation_link,
        plain_text: _,
    } = test_app.get_confirmation_links(email_request);
    println!("HTML Link: {:?}", confirmation_link);
    // Act
    reqwest::get(confirmation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Assert
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
}
