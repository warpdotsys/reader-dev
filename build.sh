#!/bin/bash
# reader-dev rust 分支（6.x）本地构建脚本
# 用法：./build.sh [web|release]
set -e

VERSION=$(grep -E '^version = ' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
echo "reader v$VERSION"

buildWeb() {
    if [ -d web ]; then
        echo "building web..."
        (cd web && (npm ci || npm install) && npm run build)
        rm -rf src/main/resources/web
        cp -r web/dist src/main/resources/web
        echo "web dist synced"
    fi
}

case $1 in
    web)
        buildWeb
        ;;
    release)
        buildWeb
        cargo build --release
        echo "done: target/release/reader"
        ;;
    *)
        cargo build
        echo "done: target/debug/reader"
        ;;
esac
