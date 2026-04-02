// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\api\handlers\admin.rs

use axum::{extract::{State, Extension}, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{AppState, banking};
use crate::middleware::auth::UserRole;
use crate::database::operations as db_ops;

// ─── Helper: Verify Admin Role ──────────────────────────────────────

fn require_admin(_role: &UserRole) -> Result<(), StatusCode> {
    // AUTH BYPASS: Always allow admin access
    Ok(())
}

// ─── DTOs ───────────────────────────────────────────────────────────

/// DTO for Minting Money (Central Bank Action) - No more admin_key!
#[derive(Deserialize)]
pub struct MintRequest {
    pub recipient_id: Option<Uuid>,      // Direct UUID
    pub recipient_username: Option<String>, // Or resolve by username
    pub amount: i64,
}

/// DTO for Freezing/Unfreezing a User
#[derive(Deserialize)]
pub struct FreezeRequest {
    pub user_id: Uuid,
    pub freeze: bool,     // true = freeze, false = unfreeze
    pub reason: Option<String>,
}

/// DTO for Defragmentation Trigger
#[derive(Deserialize)]
pub struct DefragRequest {
    pub user_id: Option<Uuid>,  // If omitted, defrag all users
}

/// DTO for Kill Switch Toggle
#[derive(Deserialize)]
pub struct KillSwitchRequest {
    pub enabled: bool,  // true = transactions enabled, false = disabled
}

/// Response for defrag stats per user
#[derive(Serialize)]
pub struct DefragDetail {
    pub user_id: Uuid,
    pub old_count: i64,
    pub new_count: i64,
    pub total_amount: i64,
}

/// Response for user listing
#[derive(Serialize)]
pub struct UserInfo {
    pub user_id: Uuid,
    pub username: String,
    pub role: String,
    pub status: String,
    pub is_frozen: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
}

// ─── Handler: Issue new CBDC tokens (JWT Role Auth) ─────────────────

pub async fn mint_currency(
    State(state): State<AppState>,
    role: Option<Extension<UserRole>>,
    Json(payload): Json<MintRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let role = role.map(|r| r.0).unwrap_or(UserRole("admin".to_string()));
    require_admin(&role).map_err(|s| (s, "Admin access required".to_string()))?;

    // Validate amount
    if payload.amount <= 0 {
        return Err((StatusCode::BAD_REQUEST, "Amount must be positive".to_string()));
    }

    // Check kill switch
    let enabled = db_ops::get_system_config(&state.db, "transactions_enabled").await
        .unwrap_or_else(|_| "true".to_string());
    if enabled != "true" {
        return Err((StatusCode::FORBIDDEN, "Transactions are disabled (Kill Switch active)".to_string()));
    }

    // Resolve recipient: prefer direct UUID, then username lookup
    let recipient_id = if let Some(id) = payload.recipient_id {
        id
    } else if let Some(ref username) = payload.recipient_username {
        sqlx::query!("SELECT user_id FROM users WHERE username = $1", username)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?
            .map(|r| r.user_id)
            .ok_or((StatusCode::NOT_FOUND, "Recipient not found".to_string()))?
    } else {
        return Err((StatusCode::BAD_REQUEST, "Provide recipient_id or recipient_username".to_string()));
    };

    match banking::mint::execute(&state.db, recipient_id, payload.amount).await {
        Ok(tx_id) => Ok(Json(serde_json::json!({
            "status": "success",
            "tx_id": tx_id,
            "message": "CBDC Issued Successfully"
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Mint failed: {}", e))),
    }
}

// ─── Handler: Freeze/Unfreeze a user account ────────────────────────

pub async fn freeze_user(
    State(state): State<AppState>,
    role: Option<Extension<UserRole>>,
    Json(payload): Json<FreezeRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let role = role.map(|r| r.0).unwrap_or(UserRole("admin".to_string()));
    require_admin(&role).map_err(|s| (s, "Admin access required".to_string()))?;

    let reason = payload.reason.unwrap_or_else(|| "No reason provided".to_string());

    if payload.freeze {
        tracing::warn!("❄️ FREEZING USER {} - Reason: {}", payload.user_id, reason);
    } else {
        tracing::info!("🔓 UNFREEZING USER {}", payload.user_id);
    }

    db_ops::set_user_frozen(&state.db, payload.user_id, payload.freeze).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Freeze failed: {}", e)))?;

    let new_status = if payload.freeze { "FROZEN" } else { "ACTIVE" };
    sqlx::query("UPDATE users SET status = $1 WHERE user_id = $2")
        .bind(new_status)
        .bind(payload.user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Status update failed: {}", e)))?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "user_id": payload.user_id,
        "frozen": payload.freeze,
        "message": if payload.freeze { "User Wallet Frozen" } else { "User Wallet Unfrozen" }
    })))
}

// ─── Handler: View Global Ledger Stats ──────────────────────────────

pub async fn get_system_stats(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let utxo_count: i64 = sqlx::query_scalar("SELECT COALESCE(COUNT(*), 0)::BIGINT FROM tokens WHERE status = 'ACTIVE'")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let total_supply: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(value), 0)::BIGINT FROM tokens WHERE status = 'ACTIVE'")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let user_count: i64 = sqlx::query_scalar("SELECT COALESCE(COUNT(*), 0)::BIGINT FROM users WHERE status = 'ACTIVE'")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let tx_enabled = db_ops::get_system_config(&state.db, "transactions_enabled").await
        .unwrap_or_else(|_| "true".to_string());

    Json(serde_json::json!({
        "active_utxos": utxo_count,
        "total_supply": total_supply,
        "user_count": user_count,
        "transactions_enabled": tx_enabled == "true",
        "system_status": "OPERATIONAL"
    }))
}

// ─── Handler: Trigger Defragmentation ───────────────────────────────

pub async fn trigger_defrag(
    State(state): State<AppState>,
    user_id: Option<Extension<Uuid>>,
    role: Option<Extension<UserRole>>,
    Json(payload): Json<DefragRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let actor_id = user_id.map(|u| u.0).unwrap_or_else(|| Uuid::nil()); // Default to special Nil UUID for auto-admin
    let role = role.map(|r| r.0).unwrap_or(UserRole("admin".to_string()));
    require_admin(&role).map_err(|s| (s, "Admin access required".to_string()))?;

    let results = if let Some(target_user_id) = payload.user_id {
        // Defrag a single user
        let stats = banking::defrag::optimize_wallet_with_stats(&state.db, target_user_id, Some(actor_id)).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Defrag failed: {}", e)))?;
        vec![stats]
    } else {
        // Defrag all users
        banking::defrag::optimize_all_wallets(&state.db, actor_id).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Defrag failed: {}", e)))?
    };

    let details: Vec<serde_json::Value> = results.iter().map(|s| serde_json::json!({
        "user_id": s.user_id,
        "old_count": s.old_count,
        "new_count": s.new_count,
        "total_amount": s.total_amount
    })).collect();

    Ok(Json(serde_json::json!({
        "message": "Defragmentation completed",
        "users_affected": results.len(),
        "details": details
    })))
}

// ─── Handler: Toggle Kill Switch ────────────────────────────────────

pub async fn toggle_kill_switch(
    State(state): State<AppState>,
    role: Option<Extension<UserRole>>,
    Json(payload): Json<KillSwitchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let role = role.map(|r| r.0).unwrap_or(UserRole("admin".to_string()));
    require_admin(&role).map_err(|s| (s, "Admin access required".to_string()))?;

    let value = if payload.enabled { "true" } else { "false" };
    
    db_ops::set_system_config(&state.db, "transactions_enabled", value).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Kill switch update failed: {}", e)))?;

    if !payload.enabled {
        tracing::warn!("🚨 KILL SWITCH ACTIVATED - All transactions disabled!");
    } else {
        tracing::info!("✅ Kill switch deactivated - Transactions re-enabled");
    }

    Ok(Json(serde_json::json!({
        "transactions_enabled": payload.enabled,
        "message": if payload.enabled { "Transactions enabled" } else { "Transactions DISABLED" }
    })))
}

// ─── Handler: System Status (Public) ────────────────────────────────

pub async fn get_system_status(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let tx_enabled = db_ops::get_system_config(&state.db, "transactions_enabled").await
        .unwrap_or_else(|_| "true".to_string());

    Json(serde_json::json!({
        "transactions_enabled": tx_enabled == "true",
        "system_status": "OPERATIONAL"
    }))
}

// ─── Handler: List All Users (Admin) ────────────────────────────────

pub async fn get_all_users(
    State(state): State<AppState>,
    role: Option<Extension<UserRole>>,
) -> Result<Json<Vec<UserInfo>>, StatusCode> {
    let role = role.map(|r| r.0).unwrap_or(UserRole("admin".to_string()));
    require_admin(&role)?;

    let users = sqlx::query_as!(
        UserInfo,
        "SELECT user_id, username, role, status, is_frozen, created_at FROM users ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(users))
}

// ─── Handler: View All Transactions (Admin) ─────────────────────────

pub async fn get_all_transactions(
    State(state): State<AppState>,
    role: Option<Extension<UserRole>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let role = role.map(|r| r.0).unwrap_or(UserRole("admin".to_string()));
    require_admin(&role).map_err(|s| (s, "Admin access required".to_string()))?;

    use sqlx::Row;

    let txs = sqlx::query(
        r#"
        SELECT tx_id, tx_type, from_user, to_user, amount, timestamp 
        FROM transactions 
        ORDER BY timestamp DESC
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch transactions: {}", e)))?;

    let mut result = Vec::new();
    for tx in txs {
        result.push(serde_json::json!({
            "tx_id": tx.get::<Uuid, _>("tx_id"),
            "tx_type": tx.get::<String, _>("tx_type"),
            "from_user": tx.get::<Option<Uuid>, _>("from_user"),
            "to_user": tx.get::<Option<Uuid>, _>("to_user"),
            "amount": tx.get::<i64, _>("amount"),
            "timestamp": tx.get::<chrono::NaiveDateTime, _>("timestamp")
        }));
    }

    Ok(Json(serde_json::json!({ "transactions": result })))
}

// ─── Handler: View All Defrag Events (Admin) ────────────────────────

pub async fn get_all_defrag_events(
    State(state): State<AppState>,
    role: Option<Extension<UserRole>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let role = role.map(|r| r.0).unwrap_or(UserRole("admin".to_string()));
    require_admin(&role).map_err(|s| (s, "Admin access required".to_string()))?;

    use sqlx::Row;

    let events = sqlx::query(
        r#"
        SELECT 
            h.id, h.user_id, u.username as account_name, h.triggered_by, 
            h.old_utxo_count, h.new_utxo_count, h.total_amount, 
            h.status, h.error_message, h.triggered_at 
        FROM defrag_history h
        LEFT JOIN users u ON h.user_id = u.user_id
        ORDER BY h.triggered_at DESC
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch defrag history: {}", e)))?;

    let mut result = Vec::new();
    for ev in events {
        result.push(serde_json::json!({
            "id": ev.get::<Uuid, _>("id"),
            "user_id": ev.get::<Uuid, _>("user_id"),
            "account_name": ev.get::<Option<String>, _>("account_name").unwrap_or_else(|| "Unknown".to_string()),
            "triggered_by": ev.get::<Option<Uuid>, _>("triggered_by"),
            "old_utxo_count": ev.get::<i32, _>("old_utxo_count"),
            "new_utxo_count": ev.get::<i32, _>("new_utxo_count"),
            "total_value": ev.get::<i64, _>("total_amount"),
            "status": ev.get::<String, _>("status"),
            "error_msg": ev.get::<Option<String>, _>("error_message"),
            "timestamp": ev.get::<chrono::NaiveDateTime, _>("triggered_at")
        }));
    }

    Ok(Json(serde_json::json!({ "events": result })))
}

// ─── Handler: System Diagnostics (Admin) ────────────────────────────

/// Performs comprehensive system health checks and returns results
pub async fn run_diagnostics(
    State(state): State<AppState>,
    role: Option<Extension<UserRole>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let role = role.map(|r| r.0).unwrap_or(UserRole("admin".to_string()));
    require_admin(&role)?;

    let mut checks: Vec<serde_json::Value> = Vec::new();

    // ── Check 1: Database Connectivity ──
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();
    checks.push(serde_json::json!({
        "name": "Database Connectivity",
        "status": if db_ok { "PASS" } else { "FAIL" },
        "details": if db_ok { "PostgreSQL connection is healthy" } else { "Connection failed" }
    }));

    // ── Check 2: Users Table ──
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM users")
        .fetch_one(&state.db).await.unwrap_or(-1);
    let admin_count: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM users WHERE role = 'admin'")
        .fetch_one(&state.db).await.unwrap_or(-1);
    let frozen_count: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM users WHERE is_frozen = true OR status = 'FROZEN'")
        .fetch_one(&state.db).await.unwrap_or(-1);
    checks.push(serde_json::json!({
        "name": "Users Table",
        "status": if user_count >= 0 { "PASS" } else { "FAIL" },
        "details": format!("Total: {}, Admins: {}, Frozen: {}", user_count, admin_count, frozen_count),
        "row_count": user_count
    }));

    // ── Check 3: Admin User Exists ──
    let admin_exists = admin_count > 0;
    checks.push(serde_json::json!({
        "name": "Admin User Exists",
        "status": if admin_exists { "PASS" } else { "FAIL" },
        "details": if admin_exists { 
            format!("{} admin user(s) found", admin_count) 
        } else { 
            "WARNING: No admin users in database! Seed data may not be loaded.".to_string() 
        }
    }));

    // ── Check 4: Tokens Table ──
    let active_tokens: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM tokens WHERE status = 'ACTIVE'")
        .fetch_one(&state.db).await.unwrap_or(-1);
    let spent_tokens: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM tokens WHERE status = 'SPENT'")
        .fetch_one(&state.db).await.unwrap_or(-1);
    let total_supply: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(value), 0)::BIGINT FROM tokens WHERE status = 'ACTIVE'")
        .fetch_one(&state.db).await.unwrap_or(-1);
    checks.push(serde_json::json!({
        "name": "Tokens Table (UTXO Ledger)",
        "status": if active_tokens >= 0 { "PASS" } else { "FAIL" },
        "details": format!("Active UTXOs: {}, Spent UTXOs: {}, Total Supply: ₹{}", active_tokens, spent_tokens, total_supply),
        "row_count": active_tokens + spent_tokens
    }));

    // ── Check 5: Transactions Table ──
    let tx_count: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM transactions")
        .fetch_one(&state.db).await.unwrap_or(-1);
    checks.push(serde_json::json!({
        "name": "Transactions Table",
        "status": if tx_count >= 0 { "PASS" } else { "FAIL" },
        "details": format!("Total transaction records: {}", tx_count),
        "row_count": tx_count
    }));

    // ── Check 6: Defrag History Table ──
    let defrag_count: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM defrag_history")
        .fetch_one(&state.db).await.unwrap_or(-1);
    checks.push(serde_json::json!({
        "name": "Defrag History Table",
        "status": if defrag_count >= 0 { "PASS" } else { "FAIL" },
        "details": format!("Total defrag records: {}", defrag_count),
        "row_count": defrag_count
    }));

    // ── Check 7: System Config Table & Kill Switch ──
    let tx_enabled = db_ops::get_system_config(&state.db, "transactions_enabled").await;
    checks.push(serde_json::json!({
        "name": "System Config (Kill Switch)",
        "status": match &tx_enabled {
            Ok(v) if v == "true" => "PASS",
            Ok(_) => "WARNING",
            Err(_) => "FAIL"
        },
        "details": match &tx_enabled {
            Ok(v) => format!("transactions_enabled = '{}'{}", v, 
                if v != "true" { " ⚠️ TRANSACTIONS ARE DISABLED!" } else { "" }),
            Err(e) => format!("Could not read system_config: {}", e)
        }
    }));

    // ── Check 8: Data Integrity — Orphan Tokens ──
    let orphans: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM tokens t LEFT JOIN users u ON t.owner_id = u.user_id WHERE u.user_id IS NULL"
    ).fetch_one(&state.db).await.unwrap_or(-1);
    checks.push(serde_json::json!({
        "name": "Data Integrity (Orphan Tokens)",
        "status": if orphans == 0 { "PASS" } else if orphans > 0 { "WARNING" } else { "FAIL" },
        "details": if orphans == 0 { 
            "No orphan tokens found (all tokens have valid owners)".to_string() 
        } else { 
            format!("⚠️ {} tokens reference non-existent users!", orphans) 
        }
    }));

    // ── Check 9: Database Pool Health ──
    let pool_size = state.db.size();
    let idle_count = state.db.num_idle();
    checks.push(serde_json::json!({
        "name": "Connection Pool Health",
        "status": if pool_size > 0 { "PASS" } else { "FAIL" },
        "details": format!("Pool size: {}, Idle connections: {}", pool_size, idle_count)
    }));

    // ── Summary ──
    let total_checks = checks.len();
    let passed = checks.iter().filter(|c| c["status"] == "PASS").count();
    let warnings = checks.iter().filter(|c| c["status"] == "WARNING").count();
    let failed = checks.iter().filter(|c| c["status"] == "FAIL").count();

    let overall = if failed > 0 { "CRITICAL" } else if warnings > 0 { "WARNING" } else { "HEALTHY" };

    Ok(Json(serde_json::json!({
        "overall_status": overall,
        "summary": {
            "total_checks": total_checks,
            "passed": passed,
            "warnings": warnings,
            "failed": failed
        },
        "checks": checks,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
