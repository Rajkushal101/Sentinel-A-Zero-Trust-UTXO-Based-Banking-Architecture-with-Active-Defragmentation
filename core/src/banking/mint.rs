// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\banking\mint.rs

use sqlx::PgPool;
use uuid::Uuid;

/// Central Bank Authority: Minting New Currency
/// In a UTXO model, this creates a token with no input (Genesis Transaction).
pub async fn execute(pool: &PgPool, recipient_id: Uuid, amount: i64) -> Result<String, String> {
    
    // Kill Switch Check — even minting is blocked when system is disabled
    let tx_enabled = crate::database::operations::get_system_config(pool, "transactions_enabled").await
        .unwrap_or_else(|_| "true".to_string());
    if tx_enabled != "true" {
        return Err("Transactions are currently disabled (Kill Switch active)".to_string());
    }

    let token_id = Uuid::new_v4();
    let tx_id = Uuid::new_v4();

    // 1. Create the Token
    sqlx::query(
        "INSERT INTO tokens (token_id, owner_id, value, status) VALUES ($1, $2, $3, 'ACTIVE')"
    )
    .bind(token_id)
    .bind(recipient_id)
    .bind(amount)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 2. Log Transaction (Audit Trail)
    sqlx::query(
        "INSERT INTO transactions (tx_id, tx_type, to_user, amount, timestamp) 
         VALUES ($1, 'MINT', $2, $3, NOW())"
    )
    .bind(tx_id)
    .bind(recipient_id)
    .bind(amount)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(tx_id.to_string())
}