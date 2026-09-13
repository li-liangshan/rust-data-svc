# rust-data-svc
Rust 2024 Axum 后端演示服务，集成 **HTTP接口 + 结构化日志 + OpenTelemetry OTLP链路追踪 + Prometheus指标**，适合微服务学习。

## ✨ Features
- Axum 高性能HTTP框架，RESTful风格接口
- 配置文件：TOML 外部配置
- 健康检查接口：`/api/v1/health/live` / `/api/v1/health/ready`
- Demo业务接口：`GET /api/v1/demo/hello?name=xxx`
- Prometheus Metrics 指标暴露（Counter自定义业务指标）
- OpenTelemetry OTLP Trace（对接Jaeger/Tempo）
- 结构化JSON日志(tracing)
- 优雅关闭(graceful shutdown，支持SIGINT/SIGTERM)
- 超时、CORS中间件

## 📁 Project Structure
```
rust-data-svc/
├── Cargo.toml
├── README.md
├── config
│   └── default.toml       # 应用配置
│   └── prod.toml       # 应用配置
├── src
│   ├── main.rs            # 程序入口，路由组装、服务启动
│   ├── app.rs             # AppState、指标注册表定义
│   ├── config.rs          # TOML配置解析
│   ├── telemetry.rs       # 日志 + OpenTelemetry链路追踪初始化
│   └── routes
│       ├── mod.rs
│       ├── health.rs      # 健康检查接口
│       ├── metrics.rs     # Prometheus指标导出接口
│       └── demo.rs        # Demo业务接口
└── target/                # 编译产物（git忽略）
```

## 📦 Dependencies
- axum: HTTP web framework
- tokio: async runtime
- serde + toml: config deserialize
- tracing + tracing-subscriber: logging
- opentelemetry + opentelemetry-otlp: distributed trace
- tracing-opentelemetry: tracing bridge for OTel
- prometheus: metrics
- tower / tower-http: middleware(cors,timeout)
- anyhow: easy error handling

## 🚀 Quick Start

### 1. Prepare config
`config/default.toml`
```toml
[service]
service_name = "rust-data-svc"
listen_addr = "127.0.0.1:8080"

[telemetry]
log_level = "INFO"
enable_jaeger = false
jaeger_endpoint = "http://127.0.0.1:4317"
otel_sample_ratio = 0.1
prometheus_path = "/metrics"
```

### 2. Run
```bash
# compile and run
cargo run
```
服务启动后监听 `127.0.0.1:8080`

### 3. Test APIs
```bash
# 健康检查
curl http://127.0.0.1:8080/api/v1/health/live

# Demo hello接口
curl "http://127.0.0.1:8080/api/v1/demo/hello?name=liliangshan"

# Prometheus metrics
curl http://127.0.0.1:8080/api/v1/metrics

# 模拟错误接口
curl http://127.0.0.1:8080/api/v1/demo/error-demo
```

## 📊 Observability
### 1. Log
输出JSON结构化日志，包含span、自定义字段，可接入ELK/Loki。

### 2. Trace (OpenTelemetry OTLP)
修改 `config/default.toml` 开启链路追踪：
```toml
enable_jaeger = true
jaeger_endpoint = "http://127.0.0.1:4317"
```
> Jaeger 需要启用 OTLP gRPC 接收端口 `4317`

### 3. Prometheus Metrics
访问 `/api/v1/metrics`，内置指标：
- `hello_request_total`: hello接口请求总数(Counter)

## ⚙ Build & Release
```bash
# debug build
cargo build

# release build
cargo build --release
```
产物路径：`target/release/rust-data-svc`

## 🛑 Graceful Shutdown
按下 `Ctrl+C` 或发送 `SIGTERM` (k8s stop)：
1. 停止接收新连接
2. Flush OTLP trace数据
3. 优雅退出

## 📝 Notes
1. OpenTelemetry version: 0.23 (OTLP, removed old jaeger agent exporter)
2. Prometheus crate: 0.13
3. Rust version: 建议 1.75+

## License
MIT
