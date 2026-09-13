# Build stage
FROM rust:1.79-slim AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends git ca-certificates && rm -rf /var/lib/apt/lists/*

RUN groupadd -r appuser && useradd -r -g appuser -u 10001 appuser

# 依赖缓存技巧
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
COPY config ./config

RUN cargo build --release
RUN chown -R appuser:appuser /build
USER appuser

# Runtime stage：最小攻击面，debian-slim，非root，只读根fs
FROM debian:bookworm-slim

WORKDIR /app

RUN groupadd -r appuser && useradd -r -g appuser -u 10001 appuser
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates tzdata && rm -rf /var/lib/apt/lists/*

COPY --from=builder --chown=appuser:appuser /build/target/release/rust-data-svc ./svc
COPY --from=builder --chown=appuser:appuser /build/config ./config

RUN chmod 500 ./svc && chmod -R 400 ./config

# 时区（国内服务建议Asia/Shanghai）
ENV TZ=Asia/Shanghai
ENV APP_ENV=prod

USER 10001
EXPOSE 8080

# k8s preStop依赖：容器内部不要额外信号处理逻辑，交给应用自己的tokio优雅关闭
CMD ["./svc"]
