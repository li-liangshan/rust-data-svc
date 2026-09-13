use std::{sync::Arc, time::Duration};

use anyhow::Result;
use axum::Router;
use tokio::{net::TcpListener, signal};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer};

mod app;
mod config;
mod error;
mod middleware;
mod routes;
mod service;
mod telemetry;

use app::AppState;
use config::AppConfig;
use middleware::request_id::set_request_id;
use routes::{health_router, metrics_router};

use crate::routes::demo_router;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = AppConfig::load()?;
    let telemetry_guard = telemetry::init_telemetry(&cfg)?;
    tracing::info!(app_env=?std::env::var("APP_ENV"), "service starting");

    let state = Arc::new(AppState::new(cfg.clone()));

    let timeout_layer = TimeoutLayer::new(Duration::from_secs(cfg.server.global_timeout_sec));
    let cors = CorsLayer::very_permissive(); // 生产按需收紧origins

    // 子路由：不带 with_state
    let api_routes = Router::new()
        .nest("/health", health_router())
        .nest("/metrics", metrics_router())
        .nest("/demo", demo_router());

    // ✅ 在最外层统一注入state
    let app = Router::new()
        .nest("/api/v1", api_routes)
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn(set_request_id))
                .layer(cors)
                .layer(timeout_layer),
        )
        .with_state(state.clone()); // 顶层一次性注入AppState

    let listener = TcpListener::bind(&cfg.server.listen_addr).await?;
    tracing::info!(addr=%cfg.server.listen_addr, "listening");

    let server = axum::serve(listener, app.into_make_service());
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        tracing::error!(err=?e, "server error");
    }

    telemetry_guard.shutdown().await;
    tracing::info!("service exited normally");
    Ok(())
}

/// 监听 SIGINT / SIGTERM(k8s terminate信号)，触发优雅关闭
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install ctrl‑c handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => tracing::info!("received SIGINT, shutting down"),
        _ = terminate => tracing::info!("received SIGTERM(k8s stop), shutting down"),
    }
}
