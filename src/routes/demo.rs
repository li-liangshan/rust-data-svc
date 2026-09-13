use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

use crate::app::AppState;

// 路由构建函数，固定返回 Router<Arc<AppState>>
pub fn demo_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/hello", get(hello_handler))
        .route("/error-demo", get(error_handler))
}

// GET查询参数结构体，serde自动解析 ?name=xxx
#[derive(Debug, Deserialize)]
pub struct HelloQuery {
    pub name: String,
}

// Handler
async fn hello_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HelloQuery>,
) -> impl IntoResponse {
    // 增加tracing span，链路追踪可以捕获这个span
    let _span = tracing::info_span!("hello_api", name = params.name.as_str()).entered();
    info!("receive hello request, name={}", params.name);

    // 业务指标：请求+1
    state.hello_request_counter.inc();
    // 使用config，编译器识别到字段被读取，警告消失
    let service_name = state.config.server.service_name.as_str();
    let resp_text = format!("[{}] Hello {} !", service_name, params.name);
    (StatusCode::OK, resp_text)
}

async fn error_handler() -> impl IntoResponse {
    let _span = tracing::error_span!("error_demo").entered();
    tracing::error!("simulate business error");
    (StatusCode::INTERNAL_SERVER_ERROR, "business error")
}
