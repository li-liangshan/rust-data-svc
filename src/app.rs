use prometheus::{Counter, Registry};
use std::sync::Arc;

use crate::config::AppConfig;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub metrics_registry: Arc<Registry>,
    // 自定义业务指标
    pub hello_request_counter: Counter,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let registry = Registry::new();

        // 定义业务Counter：hello接口总请求数
        let hello_request_counter =
            Counter::new("hello_request_total", "Total number of hello api requests").unwrap();
        registry
            .register(Box::new(hello_request_counter.clone()))
            .unwrap();
        Self {
            config: Arc::new(config),
            metrics_registry: Arc::new(registry),
            hello_request_counter,
        }
    }
}
