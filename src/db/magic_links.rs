use sqlx::SqlitePool;

use crate::{auth::User, clock::now, crypto::hash, db::users::COLUMNS, error::R};

pub(crate) async fn recent_count(db: &SqlitePool, user_id: i64) -> R<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COUNT(*) FROM magic_link_tokens WHERE user_id=? AND created_at>=?",
    )
    .bind(user_id)
    .bind(now() - 3600)
    .fetch_one(db)
    .await?)
}

pub(crate) async fn create(
    db: &SqlitePool,
    user_id: i64,
    raw_token: &str,
    ttl_minutes: i64,
) -> R<()> {
    let timestamp = now();
    sqlx::query("UPDATE magic_link_tokens SET used_at=? WHERE user_id=? AND used_at IS NULL")
        .bind(timestamp)
        .bind(user_id)
        .execute(db)
        .await?;
    sqlx::query(
        "INSERT INTO magic_link_tokens (user_id,token_hash,expires_at,created_at) VALUES (?,?,?,?)",
    )
    .bind(user_id)
    .bind(hash(raw_token))
    .bind(timestamp + ttl_minutes * 60)
    .bind(timestamp)
    .execute(db)
    .await?;
    Ok(())
}

pub(crate) async fn consume(db: &SqlitePool, raw_token: &str) -> R<Option<User>> {
    let mut transaction = db.begin().await?;
    let row: Option<(i64, i64)> = sqlx::query_as(
        "SELECT id,user_id FROM magic_link_tokens WHERE token_hash=? AND used_at IS NULL AND expires_at>?",
    )
    .bind(hash(raw_token))
    .bind(now())
    .fetch_optional(&mut *transaction)
    .await?;
    let Some((token_id, user_id)) = row else {
        return Ok(None);
    };
    let user: Option<User> = sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM users WHERE id=? AND is_active=1"
    ))
    .bind(user_id)
    .fetch_optional(&mut *transaction)
    .await?;
    let Some(user) = user else {
        return Ok(None);
    };
    if sqlx::query("UPDATE magic_link_tokens SET used_at=? WHERE id=? AND used_at IS NULL")
        .bind(now())
        .bind(token_id)
        .execute(&mut *transaction)
        .await?
        .rows_affected()
        != 1
    {
        return Ok(None);
    }
    let timestamp = now();
    sqlx::query("UPDATE users SET last_login_at=?,updated_at=? WHERE id=?")
        .bind(timestamp)
        .bind(timestamp)
        .bind(user.id)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(Some(user))
}
