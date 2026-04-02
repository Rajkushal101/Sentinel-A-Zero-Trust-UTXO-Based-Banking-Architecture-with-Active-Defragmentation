// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\security\session.rs

use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

const SESSION_DURATION: usize = 3600; // 1 Hour

/// JWT Claims - The payload of the session token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,       // Subject (User ID)
    pub role: String,    // User role: "admin" or "user"
    pub exp: usize,      // Expiration Time (Unix Timestamp)
    pub iat: usize,      // Issued At
    pub device: String,  // Device Fingerprint (Binding the token to hardware)
}

/// Generates a signed JWT for a user
/// The 'device_fp' is a hash of the user's User-Agent + IP + Screen Resolution (sent from frontend)
/// The 'role' is fetched from the database during login
pub fn create_token(user_id: Uuid, role: &str, device_fp: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id,
        role: role.to_string(),
        exp: now + SESSION_DURATION,
        iat: now,
        device: device_fp.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}