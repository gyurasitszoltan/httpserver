use sqlx::SqlitePool;

use crate::{
    auth::User,
    clock::now,
    crypto::token,
    error::{E, R},
    validation::email_of,
};

pub(crate) const COLUMNS: &str =
    "id,email,display_name,role,is_active,session_version,created_at,last_login_at";

pub(crate) async fn active(db: &SqlitePool, id: i64) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM users WHERE id=? AND is_active=1"
    ))
    .bind(id)
    .fetch_optional(db)
    .await
}

pub(crate) async fn active_by_email(db: &SqlitePool, email: &str) -> R<Option<User>> {
    Ok(sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM users WHERE email=? AND is_active=1"
    ))
    .bind(email)
    .fetch_optional(db)
    .await?)
}

pub(crate) async fn find(db: &SqlitePool, id: i64) -> R<Option<User>> {
    Ok(
        sqlx::query_as(&format!("SELECT {COLUMNS} FROM users WHERE id=?"))
            .bind(id)
            .fetch_optional(db)
            .await?,
    )
}

pub(crate) async fn bootstrap(db: &SqlitePool, email: &str) -> R<()> {
    let timestamp = now();
    sqlx::query("INSERT INTO users (email,role,is_active,session_version,created_at,updated_at) SELECT ?, 'admin',1,?,?,? WHERE NOT EXISTS (SELECT 1 FROM users WHERE role='admin' AND is_active=1)")
        .bind(email_of(email)?)
        .bind(token())
        .bind(timestamp)
        .bind(timestamp)
        .execute(db)
        .await?;
    Ok(())
}

pub(crate) async fn ensure_not_last_admin(db: &SqlitePool) -> R<()> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role='admin' AND is_active=1")
            .fetch_one(db)
            .await?;
    if count <= 1 {
        Err(E::Conflict(
            "Az utolsó aktív admin nem módosítható vagy deaktiválható.".into(),
        ))
    } else {
        Ok(())
    }
}

pub(crate) async fn list(
    db: &SqlitePool,
    query: &str,
    page: i64,
    page_size: i64,
) -> R<(Vec<User>, i64)> {
    let pattern = format!("%{}%", query.trim());
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE email LIKE ? OR COALESCE(display_name,'') LIKE ?",
    )
    .bind(&pattern)
    .bind(&pattern)
    .fetch_one(db)
    .await?;
    let users = sqlx::query_as(&format!("SELECT {COLUMNS} FROM users WHERE email LIKE ? OR COALESCE(display_name,'') LIKE ? ORDER BY id DESC LIMIT ? OFFSET ?"))
        .bind(&pattern)
        .bind(&pattern)
        .bind(page_size)
        .bind((page - 1) * page_size)
        .fetch_all(db)
        .await?;
    Ok((users, total))
}

pub(crate) async fn insert(
    db: &SqlitePool,
    email: String,
    display_name: Option<String>,
    role: String,
    is_active: bool,
) -> Result<User, sqlx::Error> {
    let timestamp = now();
    let result = sqlx::query("INSERT INTO users (email,display_name,role,is_active,session_version,created_at,updated_at) VALUES (?,?,?,?,?,?,?)")
        .bind(email)
        .bind(display_name)
        .bind(role)
        .bind(is_active as i64)
        .bind(token())
        .bind(timestamp)
        .bind(timestamp)
        .execute(db)
        .await?;
    sqlx::query_as(&format!("SELECT {COLUMNS} FROM users WHERE id=?"))
        .bind(result.last_insert_rowid())
        .fetch_one(db)
        .await
}

pub(crate) async fn update(
    db: &SqlitePool,
    id: i64,
    display_name: Option<String>,
    role: String,
    is_active: bool,
    session_version: String,
) -> R<User> {
    sqlx::query("UPDATE users SET display_name=?,role=?,is_active=?,session_version=?,updated_at=? WHERE id=?")
        .bind(display_name)
        .bind(role)
        .bind(is_active as i64)
        .bind(session_version)
        .bind(now())
        .bind(id)
        .execute(db)
        .await?;
    find(db, id).await?.ok_or(E::NotFound)
}

pub(crate) async fn deactivate(db: &SqlitePool, id: i64) -> R<()> {
    sqlx::query("UPDATE users SET is_active=0,session_version=?,updated_at=? WHERE id=?")
        .bind(token())
        .bind(now())
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}
