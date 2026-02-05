use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::LazyLock;
use url::Url;
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::{
    configuration::{DatabaseSettings, get_configuration},
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

// Ensure that the `tracing` stack is only initialized once since spawn_app is run across multiple tests
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "zero2prod=info,tower_http=warn".to_string();
    let subscriber_name = "zero2prod".to_string();

    // Set the TEST_LOG env variable if you wish to see log output. Otherwise the sink writes the logs to the void
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

#[allow(dead_code)] // Quiet errors warning about dead code. Struct used in test functions.
pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

pub struct ConfirmationLinks {
    pub html: url::Url,
    pub plain_text: url::Url,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        // Extract link from request fields
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = Url::parse(&raw_link).unwrap();
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1"); // Make sure it's localhost
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, plain_text }
    }
}

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    // Launch a mock server to mimic Postmark's API
    let email_server = MockServer::start().await;

    let host = "127.0.0.1";

    let config = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // Modify config for API testing
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.host = host.to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c.email_client.timeout_milliseconds = 250;
        c
    };

    let db_pool = configure_database(&config.database).await;

    // Build the app
    let app = Application::build(config)
        .await
        .expect("The application should build with config");
    let address = format!("http://{}:{}", host, app.port());
    let port = app.port();

    // Spawn the server
    tokio::spawn(async move {
        app.run().await.expect("server crashed");
    });

    TestApp {
        address,
        port,
        db_pool,
        email_server,
    }
}

pub async fn configure_database(db_config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&db_config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(db_config.with_db())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
