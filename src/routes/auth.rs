use axum::{Json, extract::State, http::StatusCode};
use tracing::error;

use crate::{
    auth::Session,
    crypto::token,
    db::{magic_links, users},
    dto::auth::{LinkReq, Me, Verify},
    error::{E, R},
    state::StateData,
    validation::email_of,
};

pub(crate) async fn link(
    State(state): State<StateData>,
    Json(request): Json<LinkReq>,
) -> R<StatusCode> {
    let email = email_of(&request.email)?;
    let Some(user) = users::active_by_email(&state.db, &email).await? else {
        return Ok(StatusCode::ACCEPTED);
    };
    if magic_links::recent_count(&state.db, user.id).await? >= state.cfg.per_hour {
        return Err(E::Rate);
    }
    let raw_token = token();
    magic_links::create(&state.db, user.id, &raw_token, state.cfg.ttl).await?;
    state
        .mail
        .send(
            &user.email,
            &format!("{}/auth/callback?token={raw_token}", state.cfg.base),
        )
        .await?;
    Ok(StatusCode::ACCEPTED)
}

pub(crate) async fn verify(
    State(state): State<StateData>,
    mut session: Session,
    Json(request): Json<Verify>,
) -> R<Json<Me>> {
    if request.token.len() < 32 {
        return Err(E::Unauthorized);
    }
    let user = magic_links::consume(&state.db, &request.token)
        .await?
        .ok_or(E::Unauthorized)?;
    session.login(&user).await.map_err(|error_value| {
        error!(%error_value, "session");
        E::Internal("Session létrehozása sikertelen".into())
    })?;
    Ok(Json(Me::from(&user)))
}

pub(crate) async fn logout(mut session: Session) -> R<StatusCode> {
    session
        .logout()
        .await
        .map_err(|error_value| E::Internal(error_value.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn me(session: Session) -> R<Json<Me>> {
    let user = crate::auth::user(&session)?;
    Ok(Json(Me::from(&user)))
}
