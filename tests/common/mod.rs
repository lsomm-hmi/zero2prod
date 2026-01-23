use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::LazyLock;
use uuid::Uuid;
use zero2prod::{
    configuration::{DatabaseSettings, get_configuration},
    email_client::EmailClient,
    startup::app,
    state::AppState,
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

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    // Bind to a random free port
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    let address = format!("http://{local_addr}");

    // Build the app
    let app_state = make_app_state().await;
    let db_pool = app_state.db.clone();
    let app = app(app_state);

    // Spawn the server
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server crashed");
    });

    TestApp { address, db_pool }
}

pub async fn make_app_state() -> AppState {
    // Panic if we cannot read configuration
    let mut config = get_configuration().expect("Failed to read configuration.");
    // Generate a random database name
    config.database.database_name = Uuid::new_v4().to_string();

    let db = configure_database(&config.database).await;

    let sender_email = config
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client = EmailClient::new(config.email_client.base_url, sender_email);

    let state = AppState { db, email_client };
    state
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
