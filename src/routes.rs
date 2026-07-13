use axum::{
    Router,
    routing::{get, post},
};

use crate::state::StateData;

pub mod auth;
pub mod health;
pub mod users;

pub(crate) fn api_router(state: StateData) -> Router {
    Router::new()
        .route("/healthz", get(health::health))
        .route("/api/auth/magic-link", post(auth::link))
        .route("/api/auth/verify-magic-link", post(auth::verify))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route("/api/admin/users", get(users::list).post(users::create))
        .route(
            "/api/admin/users/{id}",
            get(users::show).patch(users::update).delete(users::remove),
        )
        .with_state(state)
}
