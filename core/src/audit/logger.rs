// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\audit\logger.rs

use serde::Serialize;
use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;

/// Structured Log Entry for Forensic Analysis
#[derive(Serialize)]
struct SecurityLog {
    timestamp: String,
    level: String, // INFO, WARN, ALERT
    event: String, // "LOGIN_FAIL", "CAPTCHA_BLOCK"
    ip: String,
    user_id: Option<String>,
    details: String,
}

/// Appends a security event to a local JSON file (Immutable Append-Only Log)
pub fn log_security_event(level: &str, event: &str, ip: &str, user_id: Option<&str>, details: &str) {
    let entry = SecurityLog {
        timestamp: Utc::now().to_rfc3339(),
        level: level.to_string(),
        event: event.to_string(),
        ip: ip.to_string(),
        user_id: user_id.map(|s| s.to_string()),
        details: details.to_string(),
    };

    // Serialize to JSON Line
    if let Ok(json) = serde_json::to_string(&entry) {
        // Print to Console (for Docker logs)
        println!("[SECURITY_AUDIT] {}", json);

        // Append to File (Persistent Storage)
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open("audit_trail.jsonl")
            .and_then(|mut file| writeln!(file, "{}", json));
    }
}