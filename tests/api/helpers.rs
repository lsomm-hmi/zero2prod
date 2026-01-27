use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::LazyLock;
use uuid::Uuid;
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
    pub db_pool: PgPool,
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
}

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let host = "127.0.0.1";

    let mut config = get_configuration().expect("Failed to read configuration.");
    // Modify config for API testing
    config.database.database_name = Uuid::new_v4().to_string();
    config.application.host = host.to_string();
    config.application.port = 0;
    config.email_client.timeout_milliseconds = 250;

    let db_pool = configure_database(&config.database).await;

    // Build the app
    let app = Application::build(config)
        .await
        .expect("The application should build with config");
    let address = format!("http://{}:{}", host, app.port());

    // Spawn the server
    tokio::spawn(async move {
        app.run().await.expect("server crashed");
    });

    TestApp { address, db_pool }
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
