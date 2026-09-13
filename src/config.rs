use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct ServerConfig {
    /// 服务名称
    pub service_name: String,
    pub listen_addr: String,
    #[validate(range(min = 1, max = 300))]
    pub global_timeout_sec: u64,
    #[validate(range(min = 10, max = 2000))]
    pub max_concurrent: usize,
}

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct TelemetryConfig {
    pub log_level: String,
    pub prometheus_path: String,
    pub jaeger_endpoint: Option<String>,
    pub otel_sample_ratio: f64,
    pub enable_jaeger: bool,
}

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub telemetry: TelemetryConfig,
}

impl AppConfig {
    /// 加载优先级：default.toml < {env}.toml < 环境变量(APP_前缀，__作为.分隔)
    /// APP__SERVER__GLOBAL_TIMEOUT_SEC=15 可覆盖配置
    pub fn load() -> anyhow::Result<Self> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".into());
        let config: AppConfig = Figment::new()
            .merge(Toml::file("config/default.toml"))
            .merge(Toml::file(format!("config/{}.toml", env)))
            .merge(Env::prefixed("APP__").split("__"))
            .extract()?;
        config.validate()?;
        Ok(config)
    }
}
