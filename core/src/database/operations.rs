// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\database\operations.rs

use sqlx::PgPool;
use uuid::Uuid;
use crate::database::models::{User, DefragHistory};

// ─── Original Operations ────────────────────────────────────────────

pub async fn find_user_by_username(pool: &PgPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT user_id, username, password_hash, role, status, is_frozen, created_at FROM users WHERE username = $1",
        username
    )
    .fetch_optional(pool)
    .await
}

pub async fn is_user_active(pool: &PgPool, user_id: Uuid) -> bool {
    let result = sqlx::query!(
        "SELECT status, is_frozen FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(record)) => record.status == "ACTIVE" && !record.is_frozen,
        _ => false,
    }
}

pub async fn get_total_supply(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let rec = sqlx::query!(
        "SELECT COALESCE(SUM(value), 0)::BIGINT as total FROM tokens WHERE status = 'ACTIVE'"
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.total.unwrap_or(0))
}

// ─── Defrag History Operations ──────────────────────────────────────

pub async fn insert_defrag_history(
    pool: &PgPool,
    user_id: Uuid,
    triggered_by: Option<Uuid>,
    old_count: i32,
    new_count: i32,
    total_amount: i64,
    status: &str,
    error_message: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO defrag_history (id, user_id, triggered_by, old_utxo_count, new_utxo_count, total_amount, status, error_message)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        id,
        user_id,
        triggered_by,
        old_count,
        new_count,
        total_amount,
        status,
        error_message
    )
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn get_defrag_history_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<DefragHistory>, sqlx::Error> {
    sqlx::query_as!(
        DefragHistory,
        "SELECT id, user_id, triggered_by, triggered_at, old_utxo_count, new_utxo_count, total_amount, status, error_message
         FROM defrag_history WHERE user_id = $1 ORDER BY triggered_at DESC",
        user_id
    )
    .fetch_all(pool)
    .await
}

// ─── System Config Operations ───────────────────────────────────────

pub async fn get_system_config(pool: &PgPool, key: &str) -> Result<String, sqlx::Error> {
    let rec = sqlx::query!(
        "SELECT value FROM system_config WHERE key = $1",
        key
    )
    .fetch_one(pool)
    .await?;
    Ok(rec.value)
}

pub async fn set_system_config(pool: &PgPool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO system_config (key, value, updated_at) VALUES ($1, $2, NOW())
         ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()",
        key,
        value
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ─── User Freeze Operations ────────────────────────────────────────

pub async fn set_user_frozen(pool: &PgPool, user_id: Uuid, frozen: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE users SET is_frozen = $1 WHERE user_id = $2",
        frozen,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn is_user_frozen(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let rec = sqlx::query!(
        "SELECT is_frozen FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(rec.is_frozen)
}

// ─── User Listing ──────────────────────────────────────────────────

pub async fn get_all_users(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT user_id, username, password_hash, role, status, is_frozen, created_at FROM users ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
}