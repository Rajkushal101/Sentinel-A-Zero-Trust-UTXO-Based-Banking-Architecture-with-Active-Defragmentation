// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\middleware\rate_limit.rs

use axum::{
    extract::{Request, ConnectInfo},
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use std::{sync::{Arc, Mutex}, collections::HashMap, time::{Instant, Duration}, net::{IpAddr, SocketAddr}};
use once_cell::sync::Lazy;

// Global In-Memory Rate Limiter (Simple Token Bucket)
// In production, use Redis. For research prototype, this proves the concept.
static RATE_LIMITER: Lazy<Arc<Mutex<HashMap<IpAddr, (u32, Instant)>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

const MAX_REQUESTS: u32 = 100;
const WINDOW_SECS: u64 = 60;

/// Middleware: Anti-DDoS / Rate Limiting
pub async fn rate_limit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    
    // 1. Identify Client — prefer X-Forwarded-For (behind Nginx), fall back to ConnectInfo peer IP
    let ip: IpAddr = req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
        .or_else(|| req.extensions().get::<ConnectInfo<SocketAddr>>().map(|ci| ci.0.ip()))
        .unwrap_or_else(|| "127.0.0.1".parse().unwrap());

    // 2. Check Limit
    let is_allowed = {
        let mut limiter = RATE_LIMITER.lock().unwrap();
        let (count, last_reset) = limiter.entry(ip).or_insert((0, Instant::now()));

        if last_reset.elapsed() > Duration::from_secs(WINDOW_SECS) {
            *count = 0;
            *last_reset = Instant::now();
        }

        if *count >= MAX_REQUESTS {
            false
        } else {
            *count += 1;
            true
        }
    };

    if !is_allowed {
        tracing::warn!("⛔ Rate Limit Exceeded for IP: {}", ip);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}