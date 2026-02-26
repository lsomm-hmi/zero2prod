use crate::email_client::EmailClientError;
use crate::routes::SubscriberError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Email Client error: {0}")]
    EmailClient(#[from] EmailClientError),
    #[error("Subscriber error: {0}")]
    Subscriber(#[from] SubscriberError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            AppError::EmailClient(e) => {
                tracing::error!("Email client error: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            AppError::Subscriber(e) => {
                tracing::error!("Subscriber error: {:?}", e);
                StatusCode::BAD_REQUEST.into_response()
            }
        }
    }
}
