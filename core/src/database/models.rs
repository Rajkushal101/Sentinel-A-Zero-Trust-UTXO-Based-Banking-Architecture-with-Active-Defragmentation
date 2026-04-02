// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\database\models.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

/// User Account Model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub user_id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: String,       // 'admin', 'user', 'auditor'
    pub status: String,     // 'ACTIVE', 'FROZEN', 'SUSPENDED'
    pub is_frozen: bool,
    pub created_at: Option<NaiveDateTime>,
}

/// Transaction Audit Model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TransactionLog {
    pub tx_id: Uuid,
    pub tx_type: String, // 'MINT', 'TRANSFER', 'DEFRAG'
    pub from_user: Option<Uuid>,
    pub to_user: Option<Uuid>,
    pub amount: i64,
    pub timestamp: Option<NaiveDateTime>,
}

/// Token Model (UTXO)
/// NOTE: This duplicates `banking::utxo::Token` but is kept here for
/// generic database operations that don't need banking logic.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TokenModel {
    pub token_id: Uuid,
    pub owner_id: Uuid,
    pub value: i64,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
}

/// Defragmentation History Record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DefragHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub triggered_by: Option<Uuid>,
    pub triggered_at: Option<NaiveDateTime>,
    pub old_utxo_count: i32,
    pub new_utxo_count: i32,
    pub total_amount: i64,
    pub status: String,
    pub error_message: Option<String>,
}

/// System Configuration Key-Value Pair
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SystemConfig {
    pub key: String,
    pub value: String,
    pub updated_at: Option<NaiveDateTime>,
}