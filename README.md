# reader-dev (Rust 重写版) v6.0.0

warpdotsys/reader-dev 的 legacy 分支全量转录：后端 Rust、前端 TypeScript（Vue 2）。

## 构建

```bash
# 一键构建（web + 后端 release）
./build.sh release

# 仅后端
cargo build --release

# 仅前端（已内置构建产物到 src/main/resources/web）
cd web && npm run build
```

## 运行

两种启动形态（同一二进制）：

```bash
# CLI 模式（纯命令行服务，适合服务器/容器）
target/release/reader --port=8090 --workdir=<仓库路径>

# UI 模式（应用版：启动后自动打开默认浏览器）
target/release/reader --port=8090 --workdir=<仓库路径> --ui
```

其他参数：`--contextPath=/xxx`（URL 前缀）。

启动后浏览器访问 `http://localhost:8090`。

## 测试

```bash
# 单元测试（JS 引擎 / HTML 解析 / JSONPath / 书源解析 / okhttp）
cargo test

# 端到端测试（36 项：登录/书源/搜索/详情/目录/正文/书架/进度/分组/书签/
# 替换规则/RSS/阅读配置/WebDAV/远程导入/封面下载）
powershell -File tests/front_flow.ps1
```

## 发布

- Docker 镜像：`warpdotsys/reader`（Docker Hub）+ `ghcr.io/<repo>`，多平台 amd64 / arm64 / 386
- GitHub Release 资产：Linux / Windows × x64 / arm64 / x86（`--ui` 即应用版）
- 触发：推送 `v*` tag 或 workflow_dispatch

```bash
git tag v6.0.0 && git push origin v6.0.0
```

## 功能状态

已打通：
- 登录/用户系统、书源增删改查/禁用/批量/远程导入（真实 JSON 持久化到 `storage/data/<ns>/`）
- 书架/分组/书签/替换规则/RSS/阅读配置 CRUD
- **书源抓取链路**：搜索（单书源/多书源并发）→ 书籍详情 → 目录 → 正文（JS 规则引擎 + CSS 选择器 + JSONPath + 网络请求全真实）
- 书架刷新（网络拉取）、阅读进度、WebDAV 备份与文件操作、封面下载缓存
- UI 主题：v5.2.4 设计风格（indigo 主色、8px 圆角、浅色/深色）

## 架构

- `src/stubs.rs`：占位类型库（由编译迭代驱动逐步真实化）
- `src/runtime/`：真实运行时（boa JS 引擎、scraper HTML 解析、serde_json JSONPath、reqwest 网络）
- `src/main/java/`：Kotlin/Java 转录的 Rust 代码（包结构保留）

## 已知限制

- PDF 图片化渲染依赖系统级 PDF 渲染器（PDFBox java.awt），文本提取已真实（lopdf）；PDF 页文本可提取与搜索
- TTS 合成依赖微软 Edge/Azure 在线服务（WebSocket 连接已真实，需外网可达）
- EPUB 解析真实（zip + DOM + OPF/NCX）；复杂排版/内置字体渲染受限于 HTML 引擎
- `cache_chapter_content` 默认关闭（与原版 Kotlin 一致，可用 READER_APP_CACHECHAPTERCONTENT=true 开启）
