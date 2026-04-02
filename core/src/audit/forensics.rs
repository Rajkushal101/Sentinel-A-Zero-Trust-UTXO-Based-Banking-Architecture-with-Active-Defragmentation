// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\audit\forensics.rs

use sha2::{Sha256, Digest};
use uuid::Uuid;

/// Generates a Cryptographic Proof for a Transaction
/// Logic: Hash(Prev_Hash + Sender + Receiver + Amount + Timestamp)
/// This ensures that if a row is deleted/modified in SQL, the hash validation fails.
pub fn generate_proof(
    prev_hash: &str,
    sender: Uuid,
    receiver: Uuid,
    amount: i64,
    timestamp: i64
) -> String {
    let mut hasher = Sha256::new();
    
    hasher.update(prev_hash.as_bytes());
    hasher.update(sender.as_bytes());
    hasher.update(receiver.as_bytes());
    hasher.update(amount.to_le_bytes()); // Little Endian byte representation
    hasher.update(timestamp.to_le_bytes());

    let result = hasher.finalize();
    hex::encode(result)
}

/// Verifies if a Transaction Record matches its Hash
pub fn verify_integrity(
    record_hash: &str,
    prev_hash: &str,
    sender: Uuid,
    receiver: Uuid,
    amount: i64,
    timestamp: i64
) -> bool {
    let calculated = generate_proof(prev_hash, sender, receiver, amount, timestamp);
    calculated == record_hash
}