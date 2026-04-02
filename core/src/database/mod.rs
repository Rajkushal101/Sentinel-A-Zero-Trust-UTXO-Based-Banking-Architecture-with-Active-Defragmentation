// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\database\mod.rs

use sqlx::postgres::{PgPoolOptions, PgPool};
use std::time::Duration;

pub mod models;
pub mod operations;

/// Initialize the Database Connection Pool
/// Optimized for high concurrency financial transactions.
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(50) // High concurrency limit
        .min_connections(10) // Keep connections warm
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(60))
        .connect(database_url)
        .await?;

    // Auto-build the database tables and seed data if not built
    let init_sql = include_str!("../../../database/init.sql");
    let seed_sql = include_str!("../../../database/seed_data.sql");
    
    for statement in init_sql.split(';') {
        let stmt = statement.trim();
        if !stmt.is_empty() {
            if let Err(e) = sqlx::query(stmt).execute(&pool).await {
                tracing::warn!("Init SQL warning: {}", e);
            }
        }
    }

    for statement in seed_sql.split(';') {
        let stmt = statement.trim();
        if !stmt.is_empty() {
            if let Err(e) = sqlx::query(stmt).execute(&pool).await {
                tracing::warn!("Seed SQL warning: {}", e);
            }
        }
    }

    Ok(pool)
}