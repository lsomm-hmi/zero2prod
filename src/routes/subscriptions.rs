use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;
use crate::state::AppState;
use axum::{
    extract::{Form, State},
    http::StatusCode,
};
use chrono::Utc;
use sqlx::{Pool, Postgres};
use url::Url;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscribeFormData {
    name: String,
    email: String,
}

// Trait used for type conversions which can fail
impl TryFrom<SubscribeFormData> for NewSubscriber {
    type Error = SubscriberError;

    fn try_from(form_data: SubscribeFormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form_data.name).map_err(Self::Error::InvalidName)?;
        let email = SubscriberEmail::parse(form_data.email).map_err(Self::Error::InvalidEmail)?;
        Ok(NewSubscriber { email, name })
    }
}
#[derive(Debug)]
pub enum SubscriberError {
    InvalidName(String),
    InvalidEmail(validator::ValidationErrors),
}

impl std::fmt::Display for SubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName(msg) => write!(f, "Invalid name: {}", msg),
            Self::InvalidEmail(e) => write!(f, "Invalid email: {}", e),
        }
    }
}

#[axum::debug_handler]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(state, form_data),
    fields(
        subscriber_email = %form_data.email,
        subscriber_name = %form_data.name
    )
)]
pub async fn subscribe(
    State(state): State<AppState>,
    Form(form_data): Form<SubscribeFormData>,
) -> StatusCode {
    let db_pool = &state.db;
    let email_client = &state.email_client;

    let new_subscriber = match form_data.try_into() {
        // The TryInto trait is automatically implemented for the corresponding type used in TryFrom
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    // match insert_subscriber(db_pool, &new_subscriber).await {
    //     Ok(_) => StatusCode::OK,
    //     Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    // }
    if insert_subscriber(db_pool, &new_subscriber).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if send_confirmation_email(&email_client, new_subscriber, state.base_url)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
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
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
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

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: Url,
) -> Result<(), reqwest::Error> {
    // Dummy email to new subscriber
    // Ignoring email delivery errors for now
    let confirmation_link = base_url
        .join("subscriptions/confirm?subscription_token=mytoken")
        .expect("Failed to join confirmation path");
    let plain_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
}
