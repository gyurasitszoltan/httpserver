use axum_login::{AuthSession, AuthUser, AuthnBackend, UserId};
use sqlx::{FromRow, SqlitePool};

use crate::{
    db::users,
    error::{E, R},
};

pub(crate) type Session = AuthSession<Backend>;

#[derive(Clone, Debug, FromRow)]
pub(crate) struct User {
    pub(crate) id: i64,
    pub(crate) email: String,
    pub(crate) display_name: Option<String>,
    pub(crate) role: String,
    pub(crate) is_active: i64,
    pub(crate) session_version: String,
    pub(crate) created_at: i64,
    pub(crate) last_login_at: Option<i64>,
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> i64 {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.session_version.as_bytes()
    }
}

#[derive(Clone)]
pub(crate) struct Backend {
    db: SqlitePool,
}

impl Backend {
    pub(crate) fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

#[derive(Clone)]
pub(crate) struct Cred {
    pub(crate) id: i64,
}

impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Cred;
    type Error = sqlx::Error;

    async fn authenticate(&self, credentials: Cred) -> Result<Option<User>, sqlx::Error> {
        users::active(&self.db, credentials.id).await
    }

    async fn get_user(&self, id: &UserId<Self>) -> Result<Option<User>, sqlx::Error> {
        users::active(&self.db, *id).await
    }
}

pub(crate) fn user(session: &Session) -> R<User> {
    session.user.clone().ok_or(E::Unauthorized)
}

pub(crate) fn admin(session: &Session) -> R<User> {
    let user = user(session)?;
    if user.role == "admin" && user.is_active != 0 {
        Ok(user)
    } else {
        Err(E::Forbidden)
    }
}
