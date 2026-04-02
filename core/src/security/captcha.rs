// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\security\captcha.rs

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use aes_gcm::{
    aead::{Aead, KeyInit, AeadCore}, // FIX: Added AeadCore for generate_nonce
    Aes256Gcm, Nonce 
};
use rand::{Rng, thread_rng};
use std::time::{SystemTime, UNIX_EPOCH};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use crate::AppState;

const CAPTCHA_EXPIRY: u64 = 300; 

#[derive(Serialize)]
pub struct CaptchaResponse {
    pub svg: String,
    pub token: String,
    pub target_color: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CaptchaPayload {
    answer: String,
    exp: u64,
}

pub struct CaptchaService {
    cipher: Aes256Gcm,
}

impl CaptchaService {
    pub fn new(secret: &str) -> Self {
        let key_bytes = secret.as_bytes();
        let mut key = [0u8; 32];
        let len = std::cmp::min(key_bytes.len(), 32);
        key[..len].copy_from_slice(&key_bytes[..len]);
        
        let cipher = Aes256Gcm::new(&key.into());
        Self { cipher }
    }

    pub fn verify_token(&self, token: &str, answer: &str) -> bool {
        let data = match URL_SAFE_NO_PAD.decode(token) {
            Ok(d) => d,
            Err(_) => return false,
        };

        if data.len() < 12 { return false; }
        let (nonce, ciphertext) = data.split_at(12);
        
        let plaintext = match self.cipher.decrypt(Nonce::from_slice(nonce), ciphertext) {
            Ok(pt) => pt,
            Err(_) => return false,
        };

        let payload: CaptchaPayload = match serde_json::from_slice(&plaintext) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if now > payload.exp { return false; }

        payload.answer == answer.to_uppercase()
    }
}

pub async fn generate_handler(State(state): State<AppState>) -> Json<CaptchaResponse> {
    let mut rng = thread_rng();
    
    let pool = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let answer: String = (0..5)
        .map(|_| {
            let idx = rng.gen_range(0..pool.len());
            pool.chars().nth(idx).unwrap()
        })
        .collect();

    let target_color = "RED"; 
    let svg = format!(
        r#"<svg width="200" height="80" xmlns="http://www.w3.org/2000/svg" style="background:#f0f0f0; border-radius:4px;">
            <text x="50%" y="50%" dominant-baseline="middle" text-anchor="middle" font-family="monospace" font-size="30" fill="red" letter-spacing="5">{}</text>
            <line x1="0" y1="0" x2="200" y2="80" stroke="black" stroke-opacity="0.1" stroke-width="2"/>
            <line x1="200" y1="0" x2="0" y2="80" stroke="black" stroke-opacity="0.1" stroke-width="2"/>
           </svg>"#,
        answer
    );

    let service = CaptchaService::new(&state.config.captcha_secret);
    let payload = CaptchaPayload {
        answer: answer.clone(),
        exp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + CAPTCHA_EXPIRY,
    };
    
    let nonce = Aes256Gcm::generate_nonce(&mut rand::rngs::OsRng);
    let ciphertext = service.cipher.encrypt(&nonce, serde_json::to_vec(&payload).unwrap().as_ref()).unwrap();
    
    let mut token_bytes = nonce.to_vec();
    token_bytes.extend(ciphertext);
    let token = URL_SAFE_NO_PAD.encode(token_bytes);

    Json(CaptchaResponse {
        svg,
        token,
        target_color: target_color.to_string(),
    })
}