use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    auth::{Session, admin},
    crypto::token,
    db::users,
    dto::users::{Create, ListQ, Update, UserList, UserOut},
    error::{E, R},
    state::StateData,
    validation::{email_of, name, role},
};

pub(crate) async fn list(
    State(state): State<StateData>,
    session: Session,
    Query(query): Query<ListQ>,
) -> R<Json<UserList>> {
    admin(&session)?;
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 100);
    let (users, total) = users::list(&state.db, &query.query, page, page_size).await?;
    Ok(Json(UserList {
        items: users.into_iter().map(Into::into).collect(),
        page,
        page_size,
        total,
    }))
}

pub(crate) async fn show(
    State(state): State<StateData>,
    session: Session,
    Path(id): Path<i64>,
) -> R<Json<UserOut>> {
    admin(&session)?;
    Ok(Json(
        users::find(&state.db, id).await?.ok_or(E::NotFound)?.into(),
    ))
}

pub(crate) async fn create(
    State(state): State<StateData>,
    session: Session,
    Json(request): Json<Create>,
) -> R<(StatusCode, Json<UserOut>)> {
    admin(&session)?;
    let result = users::insert(
        &state.db,
        email_of(&request.email)?,
        name(request.display_name)?,
        role(&request.role)?,
        request.is_active.unwrap_or(true),
    )
    .await;
    match result {
        Ok(user) => Ok((StatusCode::CREATED, Json(user.into()))),
        Err(sqlx::Error::Database(error_value)) if error_value.is_unique_violation() => {
            Err(E::Conflict("Ez az e-mail cím már használatban van.".into()))
        }
        Err(error_value) => Err(error_value.into()),
    }
}

pub(crate) async fn update(
    State(state): State<StateData>,
    session: Session,
    Path(id): Path<i64>,
    Json(request): Json<Update>,
) -> R<Json<UserOut>> {
    let actor = admin(&session)?;
    let old = users::find(&state.db, id).await?.ok_or(E::NotFound)?;
    let new_role = match request.role {
        Some(value) => role(&value)?,
        None => old.role.clone(),
    };
    let is_active = request.is_active.unwrap_or(old.is_active != 0);
    if actor.id == id && !is_active {
        return Err(E::Conflict("Saját fiók nem deaktiválható.".into()));
    }
    if old.role == "admin" && old.is_active != 0 && (new_role != "admin" || !is_active) {
        users::ensure_not_last_admin(&state.db).await?;
    }
    let display_name = match request.display_name {
        Some(value) => name(value)?,
        None => old.display_name.clone(),
    };
    let session_version = if new_role != old.role || is_active != (old.is_active != 0) {
        token()
    } else {
        old.session_version
    };
    let user = users::update(
        &state.db,
        id,
        display_name,
        new_role,
        is_active,
        session_version,
    )
    .await?;
    Ok(Json(user.into()))
}

pub(crate) async fn remove(
    State(state): State<StateData>,
    session: Session,
    Path(id): Path<i64>,
) -> R<StatusCode> {
    let actor = admin(&session)?;
    let user = users::find(&state.db, id).await?.ok_or(E::NotFound)?;
    if actor.id == id {
        return Err(E::Conflict("Saját fiók nem deaktiválható.".into()));
    }
    if user.role == "admin" && user.is_active != 0 {
        users::ensure_not_last_admin(&state.db).await?;
    }
    users::deactivate(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
