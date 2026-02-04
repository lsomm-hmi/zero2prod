use crate::state::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(state, _parameters))]
pub async fn confirm(
    State(state): State<AppState>,
    Query(_parameters): Query<Parameters>,
) -> StatusCode {
    StatusCode::OK
}
