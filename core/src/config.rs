// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\config.rs

use serde::Deserialize;
use std::env;

/// Global Configuration Struct
/// Holds all environment variables loaded at startup.
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub jwt_secret: String,
    pub captcha_secret: String,
}

impl Config {
    /// Loads configuration from environment variables (.env file)
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a valid number"),
                
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set for session security"),
                
            captcha_secret: env::var("CAPTCHA_SECRET")
                .expect("CAPTCHA_SECRET must be set for stateless encryption"),
        })
    }
}