use crate::error::AppError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
#[error("Subscription confirmation error: {0}")]
pub enum SubscriptionConfirmError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Invalid subscription token")]
    SubscriptionToken,
}

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(state, parameters))]
pub async fn confirm(
    State(state): State<AppState>,
    Query(parameters): Query<Parameters>,
) -> Result<StatusCode, AppError> {
    let db_pool = &state.db;
    let id = get_subscriber_id_from_token(db_pool, &parameters.subscription_token).await?;

    match id {
        None => return Err(SubscriptionConfirmError::SubscriptionToken.into()),
        Some(subscriber_id) => {
            confirm_subscriber(db_pool, subscriber_id).await?;
        }
    }
    Ok(StatusCode::OK)
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, db_pool))]
pub async fn confirm_subscriber(
    db_pool: &Pool<Postgres>,
    subscriber_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Get subscriber_id from token",
    skip(subscription_token, db_pool)
)]
pub async fn get_subscriber_id_from_token(
    db_pool: &Pool<Postgres>,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_optional(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(result.map(|r| r.subscriber_id))
}
