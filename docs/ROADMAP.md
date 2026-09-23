# Reader-dev 维护路线

更新日期：2026-09-23。

## 当前方向

- `legacy` 是默认分支和 Java/Kotlin 混合工程的维护主线。以原始 `reader-pro-3.2.14.jar` 的可观察行为与数据格式为兼容基线，优先使用可读源码和可重复构建，而不是字节码热补丁。
- Rust 旧线和 Go 重写成果保留作历史参考，暂不继续全量重写，也不以它们作为当前版本的兼容性依据。旧 Rust 专属问题不等于已修复；仍适用于当前产品的问题继续跟踪。
- 前端现阶段保留原 JAR 的 Vue 2 界面。只有后端兼容性和主要缺陷稳定后，才评估接入 Vue 3 设计；不能直接把 Rust/master 前端视为可替换资源。
- 已发布版本、构建证据和已知限制见 `docs/releases/` 与 `reports/`。本路线图描述优先级，不代表其中每项已经交付。

## 近期优先级

1. **行为与数据兼容**：继续扩展原始 JAR、恢复版、隔离生产数据副本之间的差分，覆盖认证、用户命名空间、书源规则、书架、正文缓存、本地书、文件格式、WebDAV、SSE、下载和异常路径。将有意修复与未解释的差异分开记录。
2. **可维护源码**：逐类核对反编译或近似源码与 JAR 的差异。Kotlin 伪代码必须人工整理；不能为了通过编译而增加空实现。每个可恢复功能应有明确测试或差分证据。
3. **部署与发布安全**：GitHub 托管 runner 构建并发版，记录制品散列、已知问题和回滚方法。生产数据只在备份及隔离验证后迁移；本地构建成功不自动等于线上已经更新。
4. **当前缺陷**：继续跟踪 [#49 子目录部署](https://github.com/warpdotsys/reader-dev/issues/49)和 [#34 章节固化](https://github.com/warpdotsys/reader-dev/issues/34)。`/reader` 无尾斜杠入口已在源码中修复并通过本机黑盒检查，但 #49 涉及的反向代理、前端完整交互和生产部署尚未验收，不据此关闭。
5. **内置浏览器能力**：完成下述指纹无头浏览器选型和植入验证，目标是取消运行时必须另行部署 `remote-webview` 容器；在兼容性和资源成本达标前，不切换现有默认路径。

## 内置指纹无头浏览器（计划，尚未选型或实施）

目标是让需要 `webView` 的书源在 Reader 的一个部署单元内完成渲染：浏览器二进制和驱动随浏览器版镜像预装，由 Java/Kotlin 服务监管本地子进程；不要求用户再运行 `readerwebview` 容器，也不对公网开放浏览器控制端口。这里的“内置”不等于把浏览器塞进 JVM 进程，更不等于给普通 Chromium 改个 User-Agent 就宣称具备指纹能力。

**现有接口边界（已从当前源码确认，尚未与原 JAR 逐项差分）**：`AnalyzeUrl.useWebView` → `ReaderAdapter.getStrResponseByRemoteWebview` → `RemoteWebview.getStrResponse`。现实现向外部 `/render.html` 传 `url/html/headers/js_source/proxy/http_method/body/encode/tag/sourceRegex`，接收 HTML/文本，并把响应 `Set-Cookie` 写回对应用户命名空间。改造不得丢失 GET/POST、请求头、脚本执行、代理、字符编码、重定向和用户 Cookie 的语义；先记录真实书源的行为，再替换实现。

### 选型门槛

| 候选 | 价值 | 进入主线前必须证明的风险 |
| --- | --- | --- |
| [Playwright Java + Chromium](https://playwright.dev/java/docs/browsers) | Java/Kotlin 直接管理浏览器，依赖和进程生命周期较清晰；可作为功能/性能基线 | [设备参数模拟](https://playwright.dev/java/docs/emulation)不等于浏览器内核级指纹修改，不能单凭此方案满足“指纹浏览器”目标；须验证书源实际通过率与可检测痕迹 |
| [Camoufox](https://camoufox.com/python/usage/) | Firefox 分支提供浏览器层的指纹配置，可无头或用虚拟显示运行 | Python 启动器增加语言和打包成本；官方[跨语言服务](https://camoufox.com/python/remote-server/)标为实验性，且单浏览器服务的指纹不会按会话轮换；须验证 Java Playwright 版本匹配、隔离和重启恢复。上游也明确提示项目仍在开发，不能预设生产稳定性 |
| 其他可审计的 Chromium 指纹分支 | 如果 Camoufox 的协议或兼容性无法达标，可作为备选 PoC | 核验源码与二进制可追溯性、许可证、更新频率、Linux 架构支持和 Java 可控性；不引入只有预编译包而无法持续维护的依赖 |

不以指纹检测网站的单次“通过”作为选型结论。须在同一批合法可访问的真实书源和固定测试站点上比较成功率、HTML/脚本结果、Cookie、延迟、冷启动、常驻内存与崩溃恢复，并记录失败样本；遵守站点规则，不把绕过验证码作为验收目标。

### 改造与验收任务

- [ ] 从现有书源中提取 `webView` 用例，补齐原 JAR、当前远程服务和候选内置引擎的黑盒对照，特别覆盖 POST、代理、脚本、编码、跳转、Cookie 和多用户隔离；形成可复现但不泄露书源凭据的样本集。
- [ ] 用统一 `WebviewRenderer` 边界封装现有远程实现与本地候选实现；先允许按配置切换/回退，不改变普通非 WebView 书源和 `storage/data` 格式。明确页面复用、用户会话隔离、超时取消、进程退出与资源回收。
- [ ] 在 GitHub 托管 runner 上构建可复现的浏览器版镜像，锁定浏览器/驱动版本与 SHA-256，构建期下载并记录许可证、SBOM 和架构支持；运行时不得临时拉取浏览器。现有 `Dockerfile.source`/`Dockerfile.slim` 使用 Alpine；[Playwright 官方说明](https://playwright.dev/java/docs/docker) Firefox/WebKit 浏览器构建不支持 musl/Alpine，因此先用 glibc 基础镜像做 PoC，并验证 amd64、arm64，保留不带浏览器的轻量产物。
- [ ] 浏览器以非 root 身份运行，核对 sandbox/seccomp、字体和证书、临时目录、进程树、内存/并发上限及 SSRF/内网访问策略。若候选需要本机 WebSocket，仅绑定 loopback、使用临时受限端点，不能作为公开服务；不得为省事使用 `--no-sandbox` 作为生产默认值。
- [ ] 同时验证无头与必要时的虚拟显示模式。Camoufox 的[虚拟显示方案](https://camoufox.com/python/virtual-display/)需要 Xvfb；任何模式都必须在实际生产镜像中测试，不把宿主机 PoC 当成容器验收。
- [ ] 小流量灰度后再考虑将本地引擎设为默认；发布说明列出镜像体积、额外内存、可用架构、已知不兼容书源及回滚步骤。远程接口待存量用户迁移验证后再决定废弃，不在第一步删除。

完成条件：在**单个 Reader 容器**中，无独立 WebView 容器即可运行被选中的 WebView 书源；固定样本与真实书源的结果、Cookie/用户隔离和异常路径通过差分；空闲与并发资源预算、崩溃恢复及升级回滚均有实测证据。以上均为待办，当前版本尚不具备内置指纹浏览器。

进度记录：已在源码中加入 `WebviewRenderer`/`WebviewRequest` 边界，现有远程渲染仍为唯一默认实现；定向测试覆盖适配器参数传递与 `/render.html` 请求协议。这仅是植入准备，候选本地引擎、真实书源差分、容器镜像及生产切换均未完成。

## 后续候选项（尚未承诺）

- 在不改变旧数据语义的前提下改善 JSON 存储性能；任何 SQLite 迁移都必须具备备份、校验和可回滚路径，且不能先于兼容基线验收。
- 扩展书源规则和本地书格式时，优先在现有 Kotlin/Java 工程中做增量实现与回归测试，避免再次引入全量语言重写。
- Vue 3 设计只作为交互参考；是否移植和何时移植，以后端接口、登录与阅读链路稳定为前提。

## 验证用语

维护记录应明确区分“已从 JAR 验证”“已从近似源码推断”“已成功重建”和“尚未验证”。共同失败的第三方书源不应记为恢复版回归；仅有 HTTP 建连的 SSE 不应记为完整成功；本机通过不应写成生产已部署。
