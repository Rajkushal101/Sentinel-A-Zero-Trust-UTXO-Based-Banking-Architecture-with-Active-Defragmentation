// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\api\handlers\auth.rs

use axum::{extract::State, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use crate::{AppState, security};
use crate::security::input::InputValidator;

#[derive(Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
    pub captcha_token: String,
    pub captcha_answer: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub captcha_token: String,
    pub captcha_answer: String,
    pub device_fp: String, // Device Fingerprint
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: Uuid,
    pub role: String,  // Return role so frontend can redirect properly
}

/// Handler: Create a new Sovereign Wallet
pub async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<String>, (StatusCode, String)> {
    // 1. Input Validation (Defense in Depth)
    if !InputValidator::validate_username(&payload.username) {
        return Err((StatusCode::BAD_REQUEST, "Invalid username: must be 3-20 alphanumeric characters or underscores".to_string()));
    }
    if !InputValidator::validate_password(&payload.password) {
        return Err((StatusCode::BAD_REQUEST, "Weak password: must be 8-128 chars with uppercase, lowercase, number, and special character".to_string()));
    }

    // 2. Verify CAPTCHA (Defense Layer)
    let captcha_service = security::captcha::CaptchaService::new(&state.config.captcha_secret);
    if !captcha_service.verify_token(&payload.captcha_token, &payload.captcha_answer) {
        return Err((StatusCode::BAD_REQUEST, "Invalid CAPTCHA".to_string()));
    }

    // 3. Hash Password (Argon2id)
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Hashing failed".into()))?
        .to_string();

    // 4. Create User in Vault (role defaults to 'user' in DB schema)
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (user_id, username, password_hash, status, role) VALUES ($1, $2, $3, 'ACTIVE', 'user')")
        .bind(user_id)
        .bind(&payload.username)
        .bind(password_hash)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::CONFLICT, format!("User create failed: {}", e)))?;

    Ok(Json("Account Created".to_string()))
}

/// Handler: Secure Login
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    // 1. Input Validation
    if !InputValidator::validate_username(&payload.username) {
        return Err((StatusCode::BAD_REQUEST, "Invalid username format".to_string()));
    }

    // 2. Verify CAPTCHA (Prevent Brute Force)
    let captcha_service = security::captcha::CaptchaService::new(&state.config.captcha_secret);
    if !captcha_service.verify_token(&payload.captcha_token, &payload.captcha_answer) {
        return Err((StatusCode::FORBIDDEN, "CAPTCHA verification failed".to_string()));
    }

    // 3. Fetch User (now includes role and is_frozen)
    let user = sqlx::query!(
        "SELECT user_id, password_hash, status, role, is_frozen FROM users WHERE username = $1",
        payload.username
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB Error".into()))?
    .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".into()))?;

    // 4. Check account status
    if user.status == "FROZEN" || user.is_frozen {
        return Err((StatusCode::FORBIDDEN, "ACCOUNT FROZEN BY CENTRAL BANK".into()));
    }

    // 5. Verify Password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Hash Error".into()))?;
    
    if Argon2::default().verify_password(payload.password.as_bytes(), &parsed_hash).is_err() {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
    }

    // 6. Create Session Token (Device Bound + Role Encoded)
    let token = security::session::create_token(
        user.user_id,
        &user.role,
        &payload.device_fp, 
        &state.config.jwt_secret
    ).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token gen failed".into()))?;

    Ok(Json(LoginResponse {
        token,
        user_id: user.user_id,
        role: user.role,
    }))
}