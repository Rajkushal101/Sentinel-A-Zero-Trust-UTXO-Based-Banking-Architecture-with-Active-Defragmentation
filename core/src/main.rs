// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\core\src\main.rs

use axum::middleware::from_fn;
use dotenvy::dotenv;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod api;
mod banking;
mod security;
mod middleware;
mod audit;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: config::Config,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "sentinel_core=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Sentinel Core v5.0 (Research Edition) Initializing...");

    let config = config::Config::from_env()
        .expect("❌ Failed to load configuration. Check .env file.");

    tracing::info!("🔌 Connecting to Ledger Database...");
    let db = database::init_pool(&config.database_url)
        .await
        .expect("❌ Failed to connect to Postgres Vault");
    
    tracing::info!("✅ Database Connection Established.");

    let state = AppState {
        db,
        config: config.clone(),
    };

    let app = api::routes::create_router(state)
        // CORS: Permissive for dev. In production, Nginx handles this + the rate limiter reads X-Forwarded-For.
        .layer(CorsLayer::permissive()) 
        .layer(from_fn(crate::middleware::rate_limit::rate_limit_middleware))
        .layer(from_fn(crate::security::browser::security_headers));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("✅ Edge Gateway listening on http://{}", addr);

    // --- UDP Discovery Service (for mobile app auto-discovery) ---
    let discovery_port: u16 = std::env::var("DISCOVERY_PORT")
        .unwrap_or_else(|_| "9999".to_string())
        .parse()
        .unwrap_or(9999);
    let http_port = config.port;
    tokio::spawn(async move {
        match UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], discovery_port))).await {
            Ok(socket) => {
                // Allow receiving broadcast packets
                if let Err(e) = socket.set_broadcast(true) {
                    tracing::warn!("⚠️ Could not enable broadcast on UDP socket: {}", e);
                }
                tracing::info!("📡 Discovery service listening on UDP port {}", discovery_port);
                let mut buf = [0u8; 256];
                loop {
                    match socket.recv_from(&mut buf).await {
                        Ok((len, src)) => {
                            let msg = String::from_utf8_lossy(&buf[..len]);
                            if msg.trim() == "SENTINEL_DISCOVER" {
                                let response = format!(
                                    r#"{{"service":"sentinel-cbdc","port":{},"version":"5.0"}}"#,
                                    http_port
                                );
                                let _ = socket.send_to(response.as_bytes(), src).await;
                                tracing::debug!("📡 Discovery reply sent to {}", src);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("⚠️ Discovery socket error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("⚠️ Could not start discovery service: {}", e);
            }
        }
    });

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    // into_make_service_with_connect_info provides SocketAddr in request extensions for rate limiter
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}