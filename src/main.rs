use sqlx::PgPool;
use tokio::net::TcpListener;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::state::AppState;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber(
        "zero2prod".into(),
        "zero2prod=info,tower_http=warn".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

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
