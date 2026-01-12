use crate::configuration::Settings;
use sqlx::PgPool;

#[derive(Clone)] // Important for state sharing
pub struct AppState {
    pub db: PgPool,
    pub config: Settings,
}
