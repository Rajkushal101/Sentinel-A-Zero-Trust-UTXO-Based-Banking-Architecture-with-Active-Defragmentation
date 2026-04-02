// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\api\routes.rs

use axum::{
    routing::{get, post},
    Router, 
    middleware,
    Json,
};
use crate::{AppState, api::handlers};
use crate::middleware::auth::auth_middleware;

/// HTTP discovery endpoint — mobile apps can hit this to verify the server
async fn discover_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "sentinel-cbdc",
        "version": "5.0",
        "status": "online"
    }))
}

pub fn create_router(state: AppState) -> Router {
    
    // ─── Public Routes (No Auth) ────────────────────────────────────
    let public_routes = Router::new()
        .route("/signup", post(handlers::auth::signup))
        .route("/login", post(handlers::auth::login))
        .route("/captcha", post(crate::security::captcha::generate_handler));

    // ─── System Status (Public, read-only) ──────────────────────────
    let system_routes = Router::new()
        .route("/status", get(handlers::admin::get_system_status))
        .route("/discover", get(discover_handler));

    // ─── Wallet Routes (User Auth Required) ─────────────────────────
    let wallet_routes = Router::new()
        .route("/balance", get(handlers::wallet::get_balance))
        .route("/transfer", post(handlers::wallet::transfer_funds))
        .route("/defrag", post(handlers::wallet::defrag_wallet))
        .route("/defrag-history", get(handlers::wallet::get_defrag_history))
        .route("/transactions", get(handlers::wallet::get_transactions))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // ─── Admin Routes (Auth Removed — direct access enabled) ────────
    let admin_routes = Router::new()
        .route("/mint", post(handlers::admin::mint_currency))
        .route("/freeze", post(handlers::admin::freeze_user))
        .route("/stats", get(handlers::admin::get_system_stats))
        .route("/defrag", post(handlers::admin::trigger_defrag))
        .route("/kill-switch", post(handlers::admin::toggle_kill_switch))
        .route("/users", get(handlers::admin::get_all_users))
        .route("/history", get(handlers::admin::get_all_defrag_events))
        .route("/transactions", get(handlers::admin::get_all_transactions))
        .route("/diagnostics", get(handlers::admin::run_diagnostics));

    Router::new()
        .nest("/api/auth", public_routes)
        .nest("/api/wallet", wallet_routes)
        .nest("/api/admin", admin_routes)
        .nest("/api/system", system_routes)
        .with_state(state)
}