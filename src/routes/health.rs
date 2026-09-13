use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

use crate::app::AppState;

pub fn health_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/live", get(live))
        .route("/ready", get(ready))
}

async fn live() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn ready() -> impl IntoResponse {
    // 示例：这里可增加可选的外部依赖检查，注意超时！
    // 如果检查失败返回503，k8s停止发流量，但不会重启pod
    (StatusCode::OK, "ok")
}
