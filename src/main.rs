use zero2prod::configuration::get_configuration;
use zero2prod::startup::Application;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber(
        "zero2prod".into(),
        "zero2prod=info,tower_http=info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    // Panic if we cannot read configuration
    let config = get_configuration().expect("Failed to read configuration.");

    let app = Application::build(config).await?;

    app.run().await
}
