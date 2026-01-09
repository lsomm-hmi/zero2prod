use axum::{extract::Form, http::StatusCode};

#[derive(serde::Deserialize)]
pub struct SubscribeFormData {
    name: String,
    email: String,
}

pub async fn subscribe(Form(sub_info): Form<SubscribeFormData>) -> StatusCode {
    StatusCode::OK
}
