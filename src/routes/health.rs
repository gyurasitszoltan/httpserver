use axum::{extract::State, http::StatusCode};

use crate::{error::R, state::StateData};

pub(crate) async fn health(State(state): State<StateData>) -> R<StatusCode> {
    sqlx::query("SELECT 1").execute(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}
