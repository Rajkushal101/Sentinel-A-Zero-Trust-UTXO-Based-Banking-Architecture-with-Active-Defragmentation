// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\security\browser.rs

use axum::{
    http::{HeaderValue, header},
    middleware::Next,
    response::Response,
    extract::Request,
};

/// Middleware: Injects Security Headers into every response
/// Implements "Defense in Depth" for the frontend.
pub async fn security_headers(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // 1. CSP (Content Security Policy) - Strict
    // Only allow scripts from self. No inline styles/scripts (ideally).
    // Note: We allow 'unsafe-inline' for Tailwind/Alpine simplicity in prototype.
    let csp = "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com https://cdn.jsdelivr.net; style-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com; img-src 'self' data:; connect-src 'self' http://localhost:3000;";
    headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(csp));

    // 2. Anti-Clickjacking
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

    // 3. XSS Protection
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));

    // 4. Referrer Policy (Privacy)
    headers.insert(header::REFERRER_POLICY, HeaderValue::from_static("strict-origin-when-cross-origin"));

    response
}