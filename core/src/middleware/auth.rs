// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\middleware\auth.rs

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::AppState;

/// Wrapper type for user role, so it doesn't collide with other String extensions
#[derive(Debug, Clone)]
pub struct UserRole(pub String);

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: String,
    pub exp: usize,
    pub device: String,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    
    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];

    let decoding_key = DecodingKey::from_secret(state.config.jwt_secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Device Fingerprint Validation: Compare token's device claim with request header
    let req_fingerprint = req.headers()
        .get("X-Device-Fingerprint")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    
    if !req_fingerprint.is_empty() && token_data.claims.device != req_fingerprint {
        tracing::warn!("⚠️ Device fingerprint mismatch for user {}", token_data.claims.sub);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Insert user_id and role into request extensions for downstream handlers
    req.extensions_mut().insert(token_data.claims.sub);
    req.extensions_mut().insert(UserRole(token_data.claims.role));

    Ok(next.run(req).await)
}