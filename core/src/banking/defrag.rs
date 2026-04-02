// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\banking\defrag.rs

use sqlx::PgPool;
use uuid::Uuid;
use serde::Serialize;
use crate::database::operations as db_ops;

const MAX_OUTPUT_TOKENS: usize = 5000;

/// Defragmentation statistics returned after optimization
#[derive(Debug, Clone, Serialize)]
pub struct DefragStats {
    pub user_id: Uuid,
    pub old_count: i64,
    pub new_count: i64,
    pub total_amount: i64,
}

/// ALGORITHM C: Defragmentation Engine (Optimization)
/// Merges "Dust" (Small UTXOs) into larger notes to reduce DB size.
/// Original function preserved for backward compatibility.
pub async fn optimize_wallet(pool: &PgPool, user_id: Uuid) -> Result<String, String> {
    let stats = optimize_wallet_with_stats(pool, user_id, None).await?;
    Ok(format!("Success: Merged {} tokens into {} standard tokens (value {})", stats.old_count, stats.new_count, stats.total_amount))
}

/// Enhanced defrag that returns stats and records history.
/// `triggered_by` is the admin UUID who triggered it (None for self-service or scheduled).
pub async fn optimize_wallet_with_stats(
    pool: &PgPool,
    user_id: Uuid,
    triggered_by: Option<Uuid>,
) -> Result<DefragStats, String> {
    
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 1. Fetch active tokens
    let tokens = sqlx::query!(
        "SELECT token_id, value FROM tokens WHERE owner_id = $1 AND status = 'ACTIVE' FOR UPDATE",
        user_id
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let old_count = tokens.len() as i64;

    if tokens.len() < 2 {
        let stats = DefragStats {
            user_id,
            old_count,
            new_count: old_count,
            total_amount: tokens.first().map(|t| t.value).unwrap_or(0),
        };
        // Record even "no-op" defrag for audit completeness
        let _ = db_ops::insert_defrag_history(
            pool, user_id, triggered_by,
            old_count as i32, old_count as i32,
            stats.total_amount, "COMPLETED", None,
        ).await;
        return Ok(stats);
    }

    // 2. Calculate Total Value
    let total_value: i64 = tokens.iter().map(|t| t.value).sum();

    // 3. Build candidate consolidated output in standard denominations (like INR)
    let denominations = [2000, 500, 200, 100, 50, 20, 10, 5, 1];
    let mut remaining = total_value;
    let mut new_tokens = Vec::new();

    for &denom in &denominations {
        let count = remaining / denom;
        for _ in 0..count {
            new_tokens.push(denom);
        }
        remaining %= denom;
    }

    let new_count = new_tokens.len() as i64;

    // Safety guard: never allow defrag to increase fragmentation.
    // If denomination split is not beneficial, skip mutation.
    if new_count >= old_count {
        tx.rollback().await.map_err(|e| e.to_string())?;

        let stats = DefragStats {
            user_id,
            old_count,
            new_count: old_count,
            total_amount: total_value,
        };

        let _ = db_ops::insert_defrag_history(
            pool,
            user_id,
            triggered_by,
            old_count as i32,
            old_count as i32,
            total_value,
            "SKIPPED",
            Some("Skipped: defrag output would not reduce UTXO count"),
        ).await;

        return Ok(stats);
    }

    if new_tokens.len() > MAX_OUTPUT_TOKENS {
        tx.rollback().await.map_err(|e| e.to_string())?;

        let msg = format!(
            "Skipped: output token count {} exceeds safety limit {}",
            new_tokens.len(),
            MAX_OUTPUT_TOKENS
        );

        let _ = db_ops::insert_defrag_history(
            pool,
            user_id,
            triggered_by,
            old_count as i32,
            old_count as i32,
            total_value,
            "SKIPPED",
            Some(&msg),
        ).await;

        return Err(msg);
    }

    // 4. Burn old active tokens in one statement
    let token_ids: Vec<Uuid> = tokens.iter().map(|t| t.token_id).collect();
    sqlx::query("UPDATE tokens SET status = 'CONSOLIDATED' WHERE token_id = ANY($1)")
        .bind(&token_ids)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // 5. Insert new tokens in chunks to avoid per-row query spam
    for chunk in new_tokens.chunks(500) {
        let ids: Vec<Uuid> = (0..chunk.len()).map(|_| Uuid::new_v4()).collect();
        let vals: Vec<i64> = chunk.to_vec();

        sqlx::query(
            "INSERT INTO tokens (token_id, owner_id, value, status)
             SELECT x.token_id, $2, x.value, 'ACTIVE'
             FROM UNNEST($1::uuid[], $3::bigint[]) AS x(token_id, value)"
        )
        .bind(&ids)
        .bind(user_id)
        .bind(&vals)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 6. Record audit log
    let tx_ref = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO transactions (tx_id, tx_type, from_user, to_user, amount, timestamp) 
         VALUES ($1, 'DEFRAG', $2, $2, $3, NOW())"
    )
    .bind(tx_ref)
    .bind(user_id)
    .bind(total_value)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // 7. Commit
    tx.commit().await.map_err(|e| e.to_string())?;

    let stats = DefragStats {
        user_id,
        old_count,
        new_count,
        total_amount: total_value,
    };

    // 8. Record defrag history (outside the transaction for resilience)
    let _ = db_ops::insert_defrag_history(
        pool, user_id, triggered_by,
        old_count as i32, new_count as i32, total_value,
        "COMPLETED", None,
    ).await;

    tracing::info!(
        "🔧 Defrag: user={} merged {} UTXOs → {} (total={})",
        user_id, old_count, new_count, total_value
    );

    Ok(stats)
}

/// Defrag all users' wallets (admin bulk operation).
/// Returns stats for each user that was defragged.
pub async fn optimize_all_wallets(
    pool: &PgPool,
    triggered_by: Uuid,
) -> Result<Vec<DefragStats>, String> {
    // Get all active users
    let users = sqlx::query!("SELECT user_id FROM users WHERE status = 'ACTIVE'")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for user in users {
        match optimize_wallet_with_stats(pool, user.user_id, Some(triggered_by)).await {
            Ok(stats) => results.push(stats),
            Err(e) => {
                tracing::error!("Defrag failed for user {}: {}", user.user_id, e);
                let _ = db_ops::insert_defrag_history(
                    pool, user.user_id, Some(triggered_by),
                    0, 0, 0, "FAILED", Some(&e),
                ).await;
            }
        }
    }

    Ok(results)
}