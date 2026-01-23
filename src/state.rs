// use crate::configuration::Settings;
use crate::email_client::EmailClient;
use sqlx::PgPool;

#[derive(Clone)] // Important for state sharing
pub struct AppState {
    pub db: PgPool,
    pub email_client: EmailClient,
    // pub config: Settings, // TODO: Seeing if I really need config in State. I don't think I do.
}
