FROM rust:1.86-slim-bookworm AS builder

WORKDIR /app

# Diesel + PostgreSQL 需要 libpq 头文件和 pkg-config 才能在构建阶段完成链接
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libpq-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 先复制清单文件，尽量利用 Docker 缓存
COPY Cargo.toml Cargo.lock ./
COPY vendor ./vendor
RUN mkdir src \
    && printf "fn main() {}\n" > src/main.rs \
    && cargo build --release

# 复制真正源码后重新构建
COPY src ./src
COPY migrations ./migrations
COPY vendor ./vendor
RUN cargo build --release --bin AppUpdateService


FROM ubuntu:24.04

WORKDIR /app

# 运行期只保留必要系统库
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libpq5 tzdata \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --home-dir /app --shell /usr/sbin/nologin appuser \
    && mkdir -p /app/logs /app/app_manage \
    && chown -R appuser:appuser /app

COPY --from=builder /app/target/release/AppUpdateService /usr/local/bin/app-update-service
COPY migrations ./migrations

ENV RUST_LOG=info,access=info,salvo=info,hyper=warn,h2=warn

VOLUME ["/app/logs", "/app/app_manage"]

EXPOSE 5800

USER appuser

CMD ["/usr/local/bin/app-update-service"]
