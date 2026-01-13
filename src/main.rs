use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::state::AppState;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Add a subscriber for tracing and log bridging
    tracing_subscriber::fmt()
        // Allows you to utilize `RUST_LOG=info` when running app to set log levels
        .with_env_filter(
            EnvFilter::try_from_default_env()
                // If no RUST_LOG env variable is set, set the Env Filter manually
                .or_else(|_| EnvFilter::try_new("zero2prod=info,tower_http=warn"))
                .expect("Unable to set logging level."),
        )
        .init();

    // Panic if we cannot read configuration
    let config = get_configuration().expect("Failed to read configuration.");

    let db = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let state = AppState { db, config };

    let addr = format!("0.0.0.0:{}", state.config.application_port);
    let listener = TcpListener::bind(addr).await?;

    run(listener, state).await
}
