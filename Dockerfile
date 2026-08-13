# reader-dev rust 分支（6.x）多阶段构建
# 第一阶段：web 前端构建（bookworm 支持 linux/386；alpine 无 386 平台镜像）
FROM node:18-bookworm-slim AS web-builder
WORKDIR /web
COPY web/package.json web/package-lock.json ./
RUN npm ci || npm install
COPY web/ .
RUN npm run build

# 第二阶段：Rust 编译
FROM rust:1.85-bookworm AS rust-builder
WORKDIR /app
COPY . .
COPY --from=web-builder /web/dist /app/src/main/resources/web
RUN cargo build --release

# 第三阶段：运行镜像
FROM debian:bookworm-slim
ENV TZ=Asia/Shanghai
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*
COPY --from=rust-builder /app/target/release/reader /usr/local/bin/reader
WORKDIR /data
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/reader"]
CMD ["--port=8080"]
