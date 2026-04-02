// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\banking\transfer.rs

use sqlx::PgPool;
use uuid::Uuid;
use crate::banking::utxo;
use crate::database::operations as db_ops;

/// ALGORITHM B: Atomic UTXO Swap
/// This function guarantees that money is never created or destroyed during transfer.
pub async fn execute_transfer(
    pool: &PgPool,
    sender_id: Uuid,
    recipient_id: Uuid,
    amount: i64
) -> Result<String, String> {

    // 0a. Kill Switch Check — block all transfers if transactions are disabled
    let tx_enabled = db_ops::get_system_config(pool, "transactions_enabled").await
        .unwrap_or_else(|_| "true".to_string());
    if tx_enabled != "true" {
        return Err("Transactions are currently disabled (Kill Switch active)".to_string());
    }

    // 0b. Sender Freeze Check
    let sender_frozen = db_ops::is_user_frozen(pool, sender_id).await
        .unwrap_or(false);
    if sender_frozen {
        return Err("Your account is frozen. Contact administrator.".to_string());
    }

    // 0c. Recipient Freeze Check
    let recipient_frozen = db_ops::is_user_frozen(pool, recipient_id).await
        .unwrap_or(false);
    if recipient_frozen {
        return Err("Recipient account is frozen".to_string());
    }
    
    // 1. Coin Selection (Read Phase)
    let (inputs, total_input_value) = utxo::select_coins(pool, sender_id, amount).await?;
    let change = total_input_value - amount;

    // 2. Begin Atomic Transaction (Write Phase)
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 3. Burn Inputs (Mark as SPENT)
    for token in inputs {
        let result = sqlx::query("UPDATE tokens SET status = 'SPENT' WHERE token_id = $1 AND status = 'ACTIVE'")
            .bind(token.token_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        // Double-Spend Protection: If row wasn't updated, it was already spent!
        if result.rows_affected() == 0 {
            tx.rollback().await.unwrap_or(());
            return Err("Double-Spend Detected: Token already used".to_string());
        }
    }

    // 4. Mint Recipient Output (The Payment)
    let new_token_id = Uuid::new_v4();
    sqlx::query("INSERT INTO tokens (token_id, owner_id, value, status) VALUES ($1, $2, $3, 'ACTIVE')")
        .bind(new_token_id)
        .bind(recipient_id)
        .bind(amount)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // 5. Mint Change Output (Return to Sender)
    if change > 0 {
        let change_token_id = Uuid::new_v4();
        sqlx::query("INSERT INTO tokens (token_id, owner_id, value, status) VALUES ($1, $2, $3, 'ACTIVE')")
            .bind(change_token_id)
            .bind(sender_id)
            .bind(change)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 6. Record Audit Log (Immutable History)
    let tx_ref = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO transactions (tx_id, tx_type, from_user, to_user, amount, timestamp) 
         VALUES ($1, 'TRANSFER', $2, $3, $4, NOW())"
    )
    .bind(tx_ref)
    .bind(sender_id)
    .bind(recipient_id)
    .bind(amount)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // 7. Commit
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(tx_ref.to_string())
}