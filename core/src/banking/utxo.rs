// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\banking\utxo.rs

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

/// Represents a discrete digital token (Unspent Transaction Output)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Token {
    pub token_id: Uuid,
    pub owner_id: Uuid,
    pub value: i64,
    pub status: String, // 'ACTIVE', 'SPENT', 'BURNT'
    pub created_at: chrono::NaiveDateTime,
}

/// ALGORITHM A: Coin Selection (Knapsack-like Greedy Approach)
/// Objective: Find the best set of tokens to satisfy 'target_amount'
/// Logic: Prefer exact matches -> then largest coins -> accumulate until target met.
pub async fn select_coins(
    pool: &PgPool, 
    user_id: Uuid, 
    target_amount: i64
) -> Result<(Vec<Token>, i64), String> {
    
    // 1. Fetch all active UTXOs for the user, sorted by value (Desc)
    let available_tokens = sqlx::query_as::<_, Token>(
        "SELECT * FROM tokens WHERE owner_id = $1 AND status = 'ACTIVE' ORDER BY value DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|_| "Database error fetching coins".to_string())?;

    let mut selected = Vec::new();
    let mut current_sum = 0;

    // 2. Greedy Selection
    for token in available_tokens {
        selected.push(token.clone());
        current_sum += token.value;

        if current_sum >= target_amount {
            break;
        }
    }

    // 3. Validation
    if current_sum < target_amount {
        return Err(format!("Insufficient Funds. Available: {}, Required: {}", current_sum, target_amount));
    }

    Ok((selected, current_sum))
}