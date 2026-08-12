# reader-dev (Rust 重写版)

warpdotsys/reader-dev 的 legacy 分支全量转录：后端 Rust、前端 TypeScript（Vue 2）。

## 构建

```bash
# 后端
cargo build --release

# 前端（可选：已内置构建产物到 src/main/resources/web）
cd web && npm run build
```

## 运行

```bash
# debug
cargo run -- --port=8090 --workdir=<仓库路径>

# release
target/release/reader.exe --port=8090 --workdir=C:\path\to\repo
```

启动后浏览器访问 `http://localhost:8090`。

## 测试

```bash
# 单元测试（JS 引擎 / HTML 解析 / JSONPath / 书源解析 / okhttp）
cargo test

# API 冒烟测试（19 项：登录/书源/书架/分组 CURD/书签/RSS/替换规则/TTS/文件）
powershell -File tests/api_smoke.ps1

# 搜索链路端到端（mock 书源：保存书源 → 搜索 → 详情 → 目录 → 正文）
powershell -File tests/search_chain.ps1
```

## 功能状态

已打通：
- 登录/用户系统、书源增删改查（真实 JSON 持久化到 `storage/data/<ns>/`）
- 书架/分组/书签/替换规则/TTS 配置 CRUD
- **书源抓取链路**：搜索 → 书籍详情 → 目录 → 正文（JS 规则引擎 + CSS 选择器 + JSONPath + 网络请求全真实）
- WebDAV / 文件管理 / 定时任务（部分）

## 架构

- `src/stubs.rs`：占位类型库（由编译迭代驱动逐步真实化）
- `src/runtime/`：真实运行时（boa JS 引擎、scraper HTML 解析、serde_json JSONPath、reqwest/okhttp 网络）
- `src/main/java/`：Kotlin/Java 转录的 Rust 代码（包结构保留）

## 已知限制

- XPath 规则（AnalyzeByXPath）仍为占位
- WebDAV 上传/下载未真实化
- 部分边缘功能（EPUB/PDF 本地解析、TTS 边缘）为占位实现
