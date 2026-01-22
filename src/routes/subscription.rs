use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::state::AppState;
use axum::{
    extract::{Form, State},
    http::StatusCode,
};
use chrono::Utc;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscribeFormData {
    name: String,
    email: String,
}

#[axum::debug_handler]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(state, sub_info),
    fields(
        subscriber_email = %sub_info.email,
        subscriber_name = %sub_info.name
    )
)]
pub async fn subscribe(
    State(state): State<AppState>,
    Form(sub_info): Form<SubscribeFormData>,
) -> StatusCode {
    let db_pool = &state.db;

    let name = match SubscriberName::parse(sub_info.name) {
        Ok(name) => name,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let email = match SubscriberEmail::parse(sub_info.email) {
        Ok(email) => email,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let new_subscriber = NewSubscriber { email, name };

    match insert_subscriber(db_pool, &new_subscriber).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(db_pool, new_subscriber)
)]
pub async fn insert_subscriber(
    db_pool: &Pool<Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
