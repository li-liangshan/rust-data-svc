use anyhow::Result;
use opentelemetry::global;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};
use tracing_subscriber::layer::Layered;
use tracing_subscriber::Layer;
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    Registry,
}; // 1. 确保显式导入 Trait

use crate::config::AppConfig;

// 定义一个资源守卫，用于包装并持有你的 TracerProvider
pub struct TelemetryGuard {
    provider: Option<SdkTracerProvider>,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        // 当 Guard 被丢弃时（例如程序退出），自动优雅关闭并强行刷新未发送的 spans
        if let Some(provider) = self.provider.take() {
            if let Err(e) = provider.shutdown() {
                eprintln!("Failed to shutdown tracer provider: {:?}", e);
            }
        }
    }
}

pub fn init_telemetry(cfg: &AppConfig) -> Result<TelemetryGuard> {
    let log_filter = EnvFilter::new(cfg.telemetry.log_level.as_str());

    // JSON 日志层格式化
    let fmt_layer = fmt::layer()
        .json()
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .boxed();

    if cfg.telemetry.enable_jaeger {
        if let Some(endpoint) = &cfg.telemetry.jaeger_endpoint {
            let sampler = Sampler::TraceIdRatioBased(cfg.telemetry.otel_sample_ratio);

            // 1. 在 0.32 中，通过基础 Exporter 链式注册 gRPC 传输层和 Endpoint
            let exporter = SpanExporter::builder()
                .with_tonic() // 依赖 "grpc-tonic" 特征
                .with_endpoint(endpoint)
                .build()?; // 返回Result<SpanExporter, ...>

            // 2. 将 Exporter 传入 SdkTracerProvider 构建器，自动绑定 Tokio 异步批量导出
            let tracer_provider = SdkTracerProvider::builder()
                .with_batch_exporter(exporter) // 自动由 Tokio 运行时进行 Batch 处理
                .with_sampler(sampler)
                .build();

            tracing::info!(otlp_endpoint=%endpoint, "OTLP tracer installed successfully");

            // 3. 提取 Provider 内生成的 Tracer，并传入 tracing 桥接层
            use opentelemetry::trace::TracerProvider as _;
            let tracer = tracer_provider.tracer("app-tracer");
            let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

            // 4. 组合并注册完整的日志、链路订阅链
            let subscriber = Registry::default()
                .with(log_filter)
                .with(fmt_layer)
                .with(otel_layer);

            tracing::subscriber::set_global_default(subscriber)?;

            // 5. 将提供者设为全局，以便全局链路埋点使用
            global::set_tracer_provider(tracer_provider.clone());

            return Ok(TelemetryGuard {
                provider: Some(tracer_provider),
            });
        }
    }

    // 回退到仅有日志的无链路追踪模式
    fallback_to_logs_only(log_filter, fmt_layer)?;
    Ok(TelemetryGuard { provider: None })
}

/// 提取出的纯日志注册函数，精简代码逻辑
fn fallback_to_logs_only(
    log_filter: EnvFilter,
    fmt_layer: Box<dyn Layer<Layered<EnvFilter, Registry, Registry>> + Send + Sync + 'static>,
) -> Result<()> {
    let subscriber = Registry::default().with(log_filter).with(fmt_layer);
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

impl TelemetryGuard {
    /// 现代化的优雅关闭：替代旧的全局解绑
    pub async fn shutdown(mut self) {
        if let Some(provider) = self.provider.take() {
            // 使用 tokio 将阻塞的 OTLP 网络网络刷新操作包在后台线程运行，防止主线程死锁
            let _ = tokio::task::spawn_blocking(move || {
                let _ = provider.shutdown();
            })
            .await;
        }
        tracing::info!("telemetry shutdown complete");
    }
}
