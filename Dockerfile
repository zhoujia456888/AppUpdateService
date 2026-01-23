# 构建阶段
FROM rust:slim AS build
WORKDIR /app

# 复制依赖文件先构建依赖（利用缓存层）
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo 'fn main() { println!("Placeholder"); }' > src/main.rs && \
    cargo build --release

# 复制实际源代码并构建应用
COPY src ./src/
RUN touch src/main.rs && \
    cargo build --release

# 运行阶段使用精简基础镜像
FROM ubuntu:latest
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# 创建非root用户运行应用
RUN useradd -ms /bin/bash appuser
USER appuser
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=build /app/target/release/your_app_name ./app

# 设置容器启动命令
CMD ["./app"]