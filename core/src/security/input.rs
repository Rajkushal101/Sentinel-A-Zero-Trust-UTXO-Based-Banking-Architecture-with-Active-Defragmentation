// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\security\input.rs

use regex::Regex;
use std::sync::OnceLock;

// Compiled Regexes (Singleton Pattern for Performance)
static USERNAME_REGEX: OnceLock<Regex> = OnceLock::new();
static PASSWORD_REGEX: OnceLock<Regex> = OnceLock::new();

/// Service: Input Sanitization & Validation
/// "Defense in Depth" - Validate data before it even touches the database logic.
pub struct InputValidator;

impl InputValidator {
    /// Validates Username Format
    /// Rule: Alphanumeric + Underscores, 3-20 chars.
    pub fn validate_username(username: &str) -> bool {
        let re = USERNAME_REGEX.get_or_init(|| {
            Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap()
        });
        re.is_match(username)
    }

    /// Validates Password Complexity
    /// Rule: Min 8 chars, at least 1 uppercase, 1 lowercase, 1 number, 1 special char.
    pub fn validate_password(password: &str) -> bool {
        // Length check
        if password.len() < 8 || password.len() > 128 {
            return false;
        }

        let re = PASSWORD_REGEX.get_or_init(|| {
            // Rust regex doesn't support lookaheads directly efficiently, so we use logic checks usually,
            // but here is a simplified regex or logic check.
            // Let's use logic check for clarity and speed in Rust.
            Regex::new(r"^[\x20-\x7E]*$").unwrap() // Printable ASCII
        });

        if !re.is_match(password) { return false; }

        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_number = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        has_upper && has_lower && has_number && has_special
    }

    /// Sanitizes generic text input to prevent basic XSS
    /// (Though logic is mostly handled by Content-Security-Policy headers)
    pub fn sanitize_text(input: &str) -> String {
        input.replace('<', "&lt;")
             .replace('>', "&gt;")
             .replace('"', "&quot;")
             .replace('\'', "&#x27;")
             .trim()
             .to_string()
    }
}