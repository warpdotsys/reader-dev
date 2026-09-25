# Reader-dev 维护路线

更新日期：2026-09-25。

## 当前方向

- `legacy` 是默认分支和 Java/Kotlin 混合工程的维护主线。以原始 `reader-pro-3.2.14.jar` 的可观察行为与数据格式为兼容基线，优先使用可读源码和可重复构建，而不是字节码热补丁。
- Rust 旧线和 Go 重写成果保留作历史参考，暂不继续全量重写，也不以它们作为当前版本的兼容性依据。旧 Rust 专属问题不等于已修复；仍适用于当前产品的问题继续跟踪。
- 当前发布版继续保留原 JAR 的 Vue 2 界面作为回退；新 UI 工作转向参考 Rust `master/web-ui` 的 Vue 3 设计语言，但以 Java/Kotlin 后端为唯一接入目标。Rust 前端的功能不全，不能直接替换现有页面或照搬其后端契约。
- 已发布版本、构建证据和已知限制见 `docs/releases/` 与 `reports/`。本路线图描述优先级，不代表其中每项已经交付。

## 近期主线（2026-09-25 调整）

1. **内置浏览器**：先以固定 WebView 书源样本比较原 JAR、现有远程服务和本地候选引擎，再做单容器 PoC、进程隔离和资源预算。目标是无需另行部署 `remote-webview`；在实测达标前保留远程实现和回滚路径。下文的候选表不是选型结论。
2. **Vue 3 界面**：参考 Rust `master/web-ui` 的设计语言（配色、排版、导航和阅读体验），但页面功能与请求契约以当前 Java/Kotlin 服务和原 JAR 为准。先在独立目录建立可预览、可测试的前端，再逐页接入；完整登录、书架、搜索、阅读、书源管理通过验收前，不替换线上 Vue 2 入口。具体边界见 [Vue 3 UI 迁移核查](VUE3-UI-MIGRATION.md)。
3. **项目 Markdown 文档**：维护 README、部署、开发、兼容性与发布说明；每项写明已验证事实、待办、复现命令和已知问题，避免把 PoC、构建成功或近似源码推断写成已上线功能。文档随对应实现和测试同步更新。

持续约束：原始 JAR 与 `storage/data` 兼容仍是业务基线；缺陷修复继续配差分测试，不新增伪造空实现。GitHub 托管 runner 负责构建和发版，生产数据只在备份及隔离验证后迁移。继续跟踪 [#49 子目录部署](https://github.com/warpdotsys/reader-dev/issues/49)与 [#34 章节固化](https://github.com/warpdotsys/reader-dev/issues/34)；`/reader` 无尾斜杠入口的本机修复不等于 #49 已在生产验收。

## 内置指纹无头浏览器（普通 Chromium 基线已验证，指纹引擎未选型）

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
- [ ] 在 GitHub 托管 runner 上持续构建唯一的完整 JAR/镜像，锁定浏览器/驱动版本与 SHA-256，记录许可证、SBOM 和架构支持；运行时不得临时拉取浏览器。普通产物不再另行提供轻量版。当前基于 glibc 镜像的 amd64 合成书源测试已通过；arm64、真实书源和长期资源预算仍待验证。
- [ ] 若选 Camoufox，吸取旧 Rust 容器 [#48](https://github.com/warpdotsys/reader-dev/issues/48) 的 Python 3.12/3.13 解释器错配教训：安装、启动和健康检查必须使用同一绝对解释器路径，并在镜像测试中执行一次真实渲染。#48 属于停用的旧镜像，不作为当前 Java/Kotlin 版已存在的缺陷重开。
- [ ] 浏览器以非 root 身份运行，核对 sandbox/seccomp、字体和证书、临时目录、进程树、内存/并发上限及 SSRF/内网访问策略。若候选需要本机 WebSocket，仅绑定 loopback、使用临时受限端点，不能作为公开服务；不得为省事使用 `--no-sandbox` 作为生产默认值。
- [ ] 同时验证无头与必要时的虚拟显示模式。Camoufox 的[虚拟显示方案](https://camoufox.com/python/virtual-display/)需要 Xvfb；任何模式都必须在实际生产镜像中测试，不把宿主机 PoC 当成容器验收。
- [ ] 小流量灰度后再考虑将本地引擎设为默认；发布说明列出镜像体积、额外内存、可用架构、已知不兼容书源及回滚步骤。远程接口待存量用户迁移验证后再决定废弃，不在第一步删除。

完成条件：在**单个 Reader 容器**中，无独立 WebView 容器即可运行被选中的 WebView 书源；固定样本与真实书源的结果、Cookie/用户隔离和异常路径通过差分；空闲与并发资源预算、崩溃恢复及升级回滚均有实测证据。以上均为待办，当前版本尚不具备内置指纹浏览器。

进度记录：已在源码中加入 `WebviewRenderer`/`WebviewRequest` 边界，远程渲染仍可配置回退；定向测试覆盖适配器参数传递、`/render.html` 请求协议及按用户命名空间保存远程 Cookie。原 JAR 中 `_cookieJar` 非 URL 键被忽略的问题已在源码中有意修复，证据见 `reports/WEBVIEW-COOKIE-COMPATIBILITY.md`；原 JAR 与恢复版的三次受控 WebView 黑盒差分见 `reports/WEBVIEW-DIFF-AND-CANDIDATES.md`。独立的 [Playwright Java 功能基线 PoC](../browser-poc/README.md)在本机 Chrome 上 4 项合成页面测试通过。`LocalWebviewRenderer` 已接入 Java/Kotlin 工程，本机 3 项合成测试通过；不支持的 `sourceRegex` 和指定字符集会明确报错。[唯一完整产物的 GitHub runner 单容器测试](https://github.com/warpdotsys/reader-dev/actions/runs/36101643673)在 amd64 上验证 JAR 驱动、镜像内 Chromium、非 root/去 capabilities 条件、首页接口、3 次书源搜索与 Cookie `空 → session=alpha== → 空` 通过。当前不是指纹引擎，真实书源差分、生产环境与长期资源预算仍未完成。

安全增量（应用层，不等于完整网络隔离）：本地 Chromium 的初始目标、Playwright 可路由的 HTTP(S) 资源和 WebSocket 现经 `BrowserNetworkPolicy` 校验；Service Worker 被禁用，网络路由在 BrowserContext 层注册。默认拒绝解析到 loopback、私网、链路本地、共享地址、保留网段的目标以及 `.localhost`/`.local`/`.internal` 主机名，并拒绝 `file:` 等非网络协议。**重定向仍未由应用层封闭**：Playwright 的 route handler 对重定向链只处理首个 URL，重定向目标还需要容器/主机出口策略兜底。`READER_BROWSER_ALLOW_PRIVATE_NETWORKS=true` 是显式私网放行开关，只用于受控内网部署/本机合成 fixture，不应在不可信书源可写入的服务上启用。该 JVM 解析检查无法单独消除 DNS rebinding、代理端二次解析、重定向和 WebRTC 等通道风险；仍须在容器/主机出口侧限制对云元数据、loopback、RFC1918 和 IPv6 ULA 的访问，并用真实部署网络验证。托管镜像测试为 loopback fixture 显式启用该开关，不代表生产已验证。

## 后续候选项（尚未承诺）

- 在不改变旧数据语义的前提下改善 JSON 存储性能；任何 SQLite 迁移都必须具备备份、校验和可回滚路径，且不能先于兼容基线验收。
- 扩展书源规则和本地书格式时，优先在现有 Kotlin/Java 工程中做增量实现与回归测试，避免再次引入全量语言重写。
- Vue 3 界面的工作范围和回退门槛以上述近期主线为准；不从 Rust 前端推断 Java/Kotlin 后端已经实现同名接口。

## 验证用语

维护记录应明确区分“已从 JAR 验证”“已从近似源码推断”“已成功重建”和“尚未验证”。共同失败的第三方书源不应记为恢复版回归；仅有 HTTP 建连的 SSE 不应记为完整成功；本机通过不应写成生产已部署。
