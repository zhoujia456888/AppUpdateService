FROM rust:1.92.0-slim-bookworm AS builder

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
# 构建前自检：确保 src/main.rs 不是占位文件，避免生成“秒退”的空二进制导致容器无限重启
RUN test -f src/main.rs \
    && grep -q "tokio::main" src/main.rs \
    && grep -q "server::run" src/main.rs
RUN cargo build --release --bin AppUpdateService
RUN ls -lah /app/target/release/AppUpdateService


FROM rust:1.92.0-slim-bookworm AS runtime

WORKDIR /app

# 运行期只保留必要系统库
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Asia/Shanghai
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libpq5 tzdata passwd wget \
    && ln -snf /usr/share/zoneinfo/$TZ /etc/localtime \
    && echo $TZ > /etc/timezone \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --home-dir /app --shell /usr/sbin/nologin appuser \
    && mkdir -p /app/logs /app/app_manage \
    && chown -R appuser:appuser /app

COPY --from=builder /app/target/release/AppUpdateService /usr/local/bin/AppUpdateService
COPY migrations ./migrations

# 运行前做最小自检并确保 stdout 可见（便于排障）
RUN printf '%s\n' \
  '#!/bin/sh' \
  'set -eu' \
  'echo "[entrypoint] starting $(date -Iseconds) pid=$$"' \
  'echo "[entrypoint] user=$(id -u):$(id -g) workdir=$(pwd)"' \
  'echo "[entrypoint] database_url=${DATABASE_URL:-<unset>}"' \
  'if [ -z "${DATABASE_URL:-}" ]; then echo "[entrypoint] DATABASE_URL is unset" >&2; exit 2; fi' \
  'echo "[entrypoint] running /usr/local/bin/AppUpdateService..."' \
  'set +e' \
  '/usr/local/bin/AppUpdateService' \
  'code=$?' \
  'echo "[entrypoint] AppUpdateService exited code=$code"' \
  'exit $code' \
  > /usr/local/bin/entrypoint.sh \
  && chmod +x /usr/local/bin/entrypoint.sh

ENV RUST_LOG=info,access=info,salvo=info,hyper=warn,h2=warn

VOLUME ["/app/logs", "/app/app_manage"]

EXPOSE 5800

USER appuser

CMD ["/usr/local/bin/entrypoint.sh"]
