# reader-dev rust 分支（6.x）多阶段构建
# 方案：web 前端在构建机平台编译（node 无 386 镜像）；Rust 二进制由
# release workflow 的 build-binaries job 预编译（6 平台），此处按 TARGETARCH 复制。
# 第一阶段：web 前端构建（node 官方镜像无 linux/386 → 固定构建机平台 amd64）
FROM --platform=$BUILDPLATFORM node:18-bookworm-slim AS web-builder
WORKDIR /web
COPY web/package.json web/package-lock.json ./
RUN npm ci || npm install
COPY web/ .
RUN npm run build

# 第二阶段：运行镜像（复制预编译二进制 + web 产物，无编译）
FROM --platform=$TARGETPLATFORM debian:bookworm-slim
ARG TARGETARCH
ENV TZ=Asia/Shanghai
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*
COPY --from=web-builder /web/dist /app/src/main/resources/web
COPY bin/linux-${TARGETARCH}/reader /usr/local/bin/reader
WORKDIR /app
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/reader"]
CMD ["--port=8080"]
