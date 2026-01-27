use crate::helpers;

// Integration Testing which generates an Http server to simulate external clients
// For in-process testing, you can use Axum + Tower. Faster, but it doesn't utilize TCP/HTTP.
#[tokio::test]
async fn health_check_integration_test() {
    let test_app = helpers::spawn_app().await;

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
