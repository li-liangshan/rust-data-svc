use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use prometheus::Encoder;
use prometheus::TextEncoder;
use std::sync::Arc;

use crate::app::AppState;

pub fn metrics_router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(export_metrics))
}

async fn export_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = state.metrics_registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    (StatusCode::OK, String::from_utf8(buffer).unwrap())
}
