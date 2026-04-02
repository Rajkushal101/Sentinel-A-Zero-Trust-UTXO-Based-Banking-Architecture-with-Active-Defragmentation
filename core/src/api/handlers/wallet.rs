// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\api\handlers\wallet.rs

use axum::{extract::{State, Extension}, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{AppState, banking};
use crate::database::operations as db_ops;

#[derive(Deserialize)]
pub struct TransferRequest {
    pub recipient: String,
    pub amount: i64,
}

#[derive(Serialize)]
pub struct WalletBalance {
    pub balance: i64,
    pub utxo_count: i64,
}

pub async fn get_balance(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Json<WalletBalance> {
    
    let result = sqlx::query!(
        "SELECT 
            COALESCE(SUM(value), 0)::BIGINT as balance, 
            COUNT(*) as count 
         FROM tokens WHERE owner_id = $1 AND status = 'ACTIVE'",
        user_id
    )
    .fetch_one(&state.db)
    .await;

    let (balance, count) = match result {
        Ok(row) => (row.balance.unwrap_or(0), row.count.unwrap_or(0)),
        Err(_) => (0, 0),
    };

    Json(WalletBalance {
        balance,
        utxo_count: count,
    })
}

pub async fn transfer_funds(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<TransferRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    
    // Validate amount
    if payload.amount <= 0 {
        return Err((StatusCode::BAD_REQUEST, "Amount must be positive".to_string()));
    }

    let recipient_id = if let Ok(uuid) = Uuid::parse_str(&payload.recipient) {
        uuid
    } else {
        sqlx::query!("SELECT user_id FROM users WHERE username = $1", payload.recipient)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .map(|r| r.user_id)
            .ok_or((StatusCode::NOT_FOUND, "Recipient not found".to_string()))?
    };

    // Prevent self-transfer
    if user_id == recipient_id {
        return Err((StatusCode::BAD_REQUEST, "Cannot transfer to yourself".to_string()));
    }

    match banking::transfer::execute_transfer(&state.db, user_id, recipient_id, payload.amount).await {
        Ok(tx_id) => Ok(Json(serde_json::json!({
            "status": "success",
            "tx_id": tx_id,
            "message": "Transfer Successful"
        }))),
        Err(e) => {
            tracing::error!("Transfer failed: {}", e);
            Err((StatusCode::BAD_REQUEST, e))
        }
    }
}

/// Handler: Get defrag history for the authenticated user
pub async fn get_defrag_history(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let history = db_ops::get_defrag_history_for_user(&state.db, user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let entries: Vec<serde_json::Value> = history.iter().map(|h| serde_json::json!({
        "id": h.id,
        "triggered_at": h.triggered_at,
        "old_utxo_count": h.old_utxo_count,
        "new_utxo_count": h.new_utxo_count,
        "total_amount": h.total_amount,
        "status": h.status,
    })).collect();

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "history": entries
    })))
}

/// Handler: User self-service defrag
pub async fn defrag_wallet(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let stats = banking::defrag::optimize_wallet_with_stats(&state.db, user_id, None).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({
        "message": "Wallet optimized",
        "old_count": stats.old_count,
        "new_count": stats.new_count,
        "total_amount": stats.total_amount
    })))
}

/// Handler: Get transaction history for the authenticated user
pub async fn get_transactions(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rows = sqlx::query!(
        r#"SELECT tx_id, tx_type, from_user, to_user, amount, timestamp 
           FROM transactions 
           WHERE from_user = $1 OR to_user = $1 
           ORDER BY timestamp DESC 
           LIMIT 50"#,
        user_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let entries: Vec<serde_json::Value> = rows.iter().map(|r| {
        let direction = if r.from_user == Some(user_id) { "SENT" } else { "RECEIVED" };
        serde_json::json!({
            "tx_id": r.tx_id,
            "tx_type": r.tx_type,
            "direction": direction,
            "from_user": r.from_user,
            "to_user": r.to_user,
            "amount": r.amount,
            "timestamp": r.timestamp
        })
    }).collect();

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "transactions": entries
    })))
}