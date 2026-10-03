# Reader-dev 维护路线

更新日期：2026-10-03。

## 当前方向

- `legacy` 是默认分支和 Java/Kotlin 混合工程的维护主线。以原始 `reader-pro-3.2.14.jar` 的可观察行为与数据格式为兼容基线，优先使用可读源码和可重复构建，而不是字节码热补丁。
- Rust 旧线和 Go 重写成果保留作历史参考，暂不继续全量重写，也不以它们作为当前版本的兼容性依据。旧 Rust 专属问题不等于已修复；仍适用于当前产品的问题继续跟踪。
- 当前发布版继续保留原 JAR 的 Vue 2 界面作为回退；新 UI 工作转向参考 Rust `master/web-ui` 的 Vue 3 设计语言，但以 Java/Kotlin 后端为唯一接入目标。Rust 前端的功能不全，不能直接替换现有页面或照搬其后端契约。
- 已发布版本、构建证据和已知限制见 `docs/releases/` 与 `reports/`。本路线图描述优先级，不代表其中每项已经交付。

## 2026-09-28 当前验收快照

- **已成功重建**：候选分支 `ci/full-reader-20260926` 已把 Java/Kotlin 后端、Vue 3 前端、固定 Camoufox 浏览器及依赖放入单个完整镜像；[GitHub 托管 runner 的完整镜像作业](https://github.com/warpdotsys/reader-dev/actions/runs/36367557675)通过真实浏览器合约、镜像构建、受限容器启动、合成书源和一条公开实站 WebView 搜索。该作业不证明 ARM64、生产负载或所有真实书源兼容。具体样本、摘要和限制见[公开真实书源差分](REAL-SOURCE-DIFF-2026-09-28.md)。
- **已从原始 JAR 验证**：同一公开站点的静态目录样本在原 JAR 和当前本机恢复构建中均返回 10 本书；HTTP 状态、`ReturnData` 及规范化数据摘要一致。该样本未启用 WebView，不能与上一条拼成原 JAR／旧远程服务／Camoufox 三方等价。
- **尚未验证**：旧远程 WebView 服务当前不在生产主机运行，需另行建立可追溯、隔离的同条件对照；登录态、真实脚本规则、长期并发、ARM64、生产数据与公网部署也未通过验收。`legacy` 默认分支和生产 `read.medwarp.cn` 仍未切到该候选，不创建正式发布标签。候选三道门禁、已知问题和回滚边界见[验收记录](CANDIDATE-2026-09-28.md)。下文 2026-09-25/26 的进度段落保留历史语境，不能代替此快照。
- **已排除一条错误对照**：公共镜像站的 3.2.14 标签并非本地原 JAR 的相同字节，大小和 SHA-256 均不同；不能用它补齐旧服务的三方差分。见[来源核验](ORIGINAL-JAR-PROVENANCE-2026-09-28.md)。
- **旧实现参考（不等于生产实例）**：固定摘要 `remote-webview:3.2.0` 在隔离 [托管 runner](https://github.com/warpdotsys/reader-dev/actions/runs/36437640038)上完成合成 GET/POST、公开目录，以及脚本返回字符串、对象、数组、数字的探针。最初无返回值的脚本探针不符合该实现的接口约定，不能作为故障证据。见[旧远程 WebView 参考记录](ARCHIVED-REMOTE-WEBVIEW-2026-09-28.md)。原 JAR／旧参考／Camoufox 尚未同条件贯通。

**2026-09-29 追加**：当前候选 `e128bcb8` 的 Java/Kotlin、Vue 3 与完整单镜像三道托管门禁均通过；[同一次手动配对运行](https://github.com/warpdotsys/reader-dev/actions/runs/36529322389)对相同公开 URL 得到旧参考镜像 DOM 计数 10、内置 Camoufox 经 Reader 搜索解析 10，采样相隔 433.381 秒。它不是原 JAR 或原生产实例对照，不能抵消上面的三方与真实登录书源未验收项；详见[公开实站记录](REAL-SOURCE-DIFF-2026-09-28.md)。

**2026-09-29 更强样本**：[配对作业 36531865466](https://github.com/warpdotsys/reader-dev/actions/runs/36531865466)不仅数量同为 10，还得到相同书名/URL 投影 SHA-256；相隔 237.914 秒。此前一次附加纯数字脚本不稳定导致整次作业红灯，已移除并重新托管验证。原 JAR 和原生产远程服务仍未加入此配对，真实登录书源与完整兼容验收继续未完成。

**2026-09-29 原 JAR 隔离复验**：另一次合成 Cookie／脚本字段测试在仅回环网卡的 Linux 网络命名空间里实际运行本地原始 JAR 与当前恢复构建，四次搜索均成功，`js_source` 字段相同；恢复版回放并删除 Cookie，原 JAR 未回放。详见[运行边界及原始报告](ORIGINAL-JAR-NETNS-WEBVIEW-DIFF-2026-09-29.md)。这不是上条公开站点配对的第三方，也未覆盖原生产远程 WebView、真实脚本执行或线上 Vue 3 登录。

**同日 POST 扩展**：上述隔离双 JAR 夹具又增加第 5 次合成书源搜索，两边向 `/render.html` 传递的 `http_method=POST`、请求体、自定义头与脚本字段相同，HTTP 结果均成功；[独立 JSON 和严格边界](ORIGINAL-JAR-NETNS-WEBVIEW-DIFF-2026-09-29.md#同日追加post-请求形态实测)已记录。夹具未真正访问目标站，不能把请求字段等同于远程服务／内置浏览器的实际 POST 行为。

**2026-10-03 入库的实际旧引擎对照**：2026-09-29 在 `--network none` 的固定摘要历史 WebKit 容器网络中，原始 JAR 与恢复构建各完成 5 次目标搜索；4 次 GET、1 次 POST 的目标方法/请求体/测试头相同，脚本书名改写被两侧成功解析。目标 Cookie 两侧全为空，与模拟 `/render.html` 的 Cookie 回放差异属于不同观测位置。[结果、脚本断言、复现命令和原始 JSON](ORIGINAL-JAR-NETNS-WEBVIEW-DIFF-2026-09-29.md#同日追加实际历史-webkit-服务的双-jar-差分)已保存。此轮仍未加入 Camoufox、真实登录书源或原生产远程实例；正式发布和部署门槛继续未完成。

**2026-10-03 完整镜像复验**：`024da9d7` 的 Java/Kotlin、Vue 3 和完整镜像三道托管 workflow 全部通过；完整镜像新增的脚本书名改写与目标 POST 字段断言通过，4 个用户同时请求结果均成功。cgroup 内存峰值约 772 MiB、PIDs（含线程）峰值 177，OOM 和 PIDs 限额触发计数均为 0；[原始制品与资源边界](CANDIDATE-2026-09-28.md#当前验收快照2026-10-03)已记录。这是短时有限预算样本，长期吞吐、真实需登录书源和最终部署继续待验收。

**同日公开样本复验**：新[配对作业 37090342192](https://github.com/warpdotsys/reader-dev/actions/runs/37090342192)在 `024da9d7` 上成功；旧参考与内置候选的 10 项书名/URL 投影摘要相同。含公开页负载的镜像内存峰值约 794 MiB、PIDs（含线程）峰值 199，OOM/限额事件为 0。[证据与 416.662 秒采样差](REAL-SOURCE-DIFF-2026-09-28.md#2026-10-03-当前候选公开页复验)已记录。公开列表较 2026-09-29 改变；本次没有把早期原 JAR 的结果拼接为第三侧。

**同日登录边界增量**：`c5879535` 的三道托管门禁全部通过；[Vue 3 作业](https://github.com/warpdotsys/reader-dev/actions/runs/37092091245)下载的核心 JUnit 为 8 项、0 跳过/失败/错误。新增错误密码拒绝后直接重试、取消“记住我”、tab-local token 刷新及退出后拒绝受保护入口的断言，见[原始制品摘要](evidence/vue3-login-c5879535-2026-10-03.json)。仍为隔离测试账号，不是现有生产用户验收。新增[原生 ARM64 托管验收入口](NATIVE-ARM64-ACCEPTANCE-2026-10-03.md)，在实际运行通过前不把 ARM64 列为已交付。

## 近期主线（2026-09-26 调整）

1. **内置浏览器**：先以固定 WebView 书源样本比较原 JAR、现有远程服务和本地候选引擎，再做单容器 PoC、进程隔离和资源预算。目标是无需另行部署 `remote-webview`；在实测达标前保留远程实现和回滚路径。下文的候选表不是选型结论。
2. **Vue 3 界面**：参考 Rust `master/web-ui` 的设计语言（配色、排版、导航和阅读体验），但页面功能与请求契约以当前 Java/Kotlin 服务和原 JAR 为准。先在独立目录建立可预览、可测试的前端，再逐页接入；完整登录、书架、搜索、阅读、书源管理通过验收前，不替换线上 Vue 2 入口。具体边界见 [Vue 3 UI 迁移核查](VUE3-UI-MIGRATION.md)。
3. **项目 Markdown 文档**：维护 README、部署、开发、兼容性与发布说明；每项写明已验证事实、待办、复现命令和已知问题，避免把 PoC、构建成功或近似源码推断写成已上线功能。文档随对应实现和测试同步更新。

持续约束：原始 JAR 与 `storage/data` 兼容仍是业务基线；缺陷修复继续配差分测试，不新增伪造空实现。GitHub 托管 runner 负责构建和发版，生产数据只在备份及隔离验证后迁移。继续跟踪 [#49 子目录部署](https://github.com/warpdotsys/reader-dev/issues/49)与 [#34 章节固化](https://github.com/warpdotsys/reader-dev/issues/34)；`/reader` 无尾斜杠入口的本机修复不等于 #49 已在生产验收。

## 最终完整版本发布与部署门槛（2026-09-26 更新）

本节也是当前交付目标的追加完成定义：先完成并验收内置浏览器、Vue 3 全功能界面、兼容性回归、项目文档及托管 CI；然后统一提升后端与所有 Web 版本号、构建并发布 GitHub Release、GHCR 和 Docker Hub 镜像；最后在 `cdn.medwarp.cn` 部署、验证并记录回滚。只有这些阶段都完成且有证据，才把整个目标视为完成。

本节是浏览器、Vue 3 主界面、兼容性回归和项目文档均达到各自验收条件之后的最后阶段；不得为了提前发版而降低上述验收标准。发布前重新查询 GitHub 最新正式 Release，并选择严格高于它的统一产品版本；本机 GitHub CLI 于 2026-09-26 查到的最新正式版为 `v6.0.45`，恢复线 `v4.0.7-restored.8` 是预发布，最终版本号须在实际发布时再次核实，不能把现在的快照当成永久版本结论。

- [ ] 盘点并统一后端 `build.gradle.kts`、原版前端 `web/package.json`、Vue 3 前端 `web-vue3/package.json`、嵌入静态资源及容器标签的版本声明；全部提升到高于发布前最新正式 Release 的版本。2026-09-28 候选三者已暂时统一为 `4.0.7`，但这不是正式发布版本；发布前仍须重新查询最新正式 Release 并统一提升，不能以候选号直接发版。
- [ ] 在 GitHub 托管 runner 上执行完整测试、前后端构建、镜像构建和制品校验；发布说明采用项目维护者口吻，明确实际验证范围、已知问题、升级/数据兼容注意事项及回滚方法。
- [ ] 创建正式 GitHub Release 并附带可校验的构建产物；发布带版本标签的 GHCR 镜像和 Docker Hub 镜像，并验证两个 registry 中的摘要与构建产物一致。镜像仓库名、可见性、凭据和不可变标签策略须先核实，不把本地 `docker save` 归档误报为已推送镜像。
- [ ] 将已验证的产物部署到用户指定的 `cdn.medwarp.cn` 主机，先保留现有容器、配置和存储的可回滚副本，再执行健康检查及公网冒烟测试。`cdn.medwarp.cn` 在此处是部署主机；当前应用公网域名仍配置为 `read.medwarp.cn`，不得擅自把两者混为一谈或更改应用域名。
- [ ] 若需要导入数据，先确认数据来源和目标命名空间，备份目标 `storage/data` 并在隔离副本验证格式、用户归属和读写；未经校验不得覆盖现有生产数据。部署和数据导入分别记录证据、结果与回滚步骤。

**当前发布链路状态（源码候选，尚未实际发布）**：正式版标签校验、GitHub 托管 runner 的 Vue 3 构建、GHCR/Docker Hub 双镜像推送、`cdn.medwarp.cn` 部署与故障回滚已写入工作流；PyPI 固定依赖的 wheel SHA-256 已写入锁文件。候选分支的 Java/Kotlin、Vue 3 与单容器浏览器三道测试工作流已由托管 runner 实跑通过，但带稳定标签的 `release.yml` 尚未执行；双架构镜像、镜像仓库凭据、生产 SSH/回滚分支仍需正式发布时验证。当前未升版本、未创建 Release、未推送新镜像、未部署候选，不能把测试工作流的绿灯说成上线证据。

## 内置指纹无头浏览器（历史计划与阶段记录；当前状态见上方快照）

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

完成条件：在**单个 Reader 容器**中，无独立 WebView 容器即可运行被选中的 WebView 书源；固定样本与真实书源的结果、Cookie/用户隔离和异常路径通过差分；空闲与并发资源预算、崩溃恢复及升级回滚均有实测证据。候选镜像已满足单容器启动和部分样本，但整个完成条件尚未达到，当前生产版本也未切换到该候选。

进度记录：已在源码中加入 `WebviewRenderer`/`WebviewRequest` 边界，远程渲染仍可配置回退；定向测试覆盖适配器参数传递、`/render.html` 请求协议及按用户命名空间保存远程 Cookie。原 JAR 中 `_cookieJar` 非 URL 键被忽略的问题已在源码中有意修复，证据见 `reports/WEBVIEW-COOKIE-COMPATIBILITY.md`；原 JAR 与恢复版的三次受控 WebView 黑盒差分见 `reports/WEBVIEW-DIFF-AND-CANDIDATES.md`。独立的 [Playwright Java 功能基线 PoC](../browser-poc/README.md)在本机 Chrome 上 4 项合成页面测试通过。`LocalWebviewRenderer` 已接入 Java/Kotlin 工程；HTTP 与 SOCKS4/5 书源代理经本地出口代理转发，单元测试验证目标 IP 固定、HTTP/SOCKS5 认证以及凭据不泄漏到源站。[GitHub 托管 runner 36131976893](https://github.com/warpdotsys/reader-dev/actions/runs/36131976893)以真实 Chromium 验证了 GET/POST、Cookie 用户隔离、JS 子资源、分块 SSE 以及跨重定向拦截 loopback；同一 workflow 还构建单容器完整镜像，并通过首页、合成书源搜索和 Cookie 序列 `空 → 空 → session=alpha== → 空` 烟测。Java/Kotlin CI 36131976780 通过。本机全量 Gradle 测试为 52 项、0 失败，7 项因本机无 Chromium 而跳过；托管 runner 补跑了浏览器测试。仍不支持 `sourceRegex` 和非 UTF-8 `encode`；当前不是指纹引擎，真实书源兼容差分、生产网络隔离、ARM64、长期并发资源预算仍未完成。

安全增量（应用层，不等于完整网络隔离）：Chromium 的 HTTP、HTTPS CONNECT 与 WebSocket 经每次渲染独立的 loopback 出口代理；对每个请求重新解析、检查整个 DNS 答案集并连接到已校验的 IP，重定向和子资源因此也经过相同规则。上游 HTTP/SOCKS 代理同样接收固定 IP 目标；书源代理端点本身也受网络策略约束。Playwright 路由仍保留作纵深防御，Service Worker 禁用，Chromium 禁用 QUIC 并限制非代理 WebRTC UDP。默认拒绝 loopback、私网、链路本地、共享地址、保留网段以及 `.localhost`/`.local`/`.internal` 主机名，并拒绝 `file:` 等非网络协议。**回归证据**：[runner 36131976893](https://github.com/warpdotsys/reader-dev/actions/runs/36131976893)中的真实 Chromium 重定向 fixture 从允许的 `127.0.0.1` 跳转到 `localhost` 后被出口代理拒绝，目标端命中数为 0；JS 子资源与分块 SSE 同时通过。`READER_BROWSER_ALLOW_PRIVATE_NETWORKS=true` 仅为受控合成 fixture/自管内网显式放行；托管镜像烟测使用该开关，不证明公网生产隔离。应用层代理仍不等于内核级网络隔离，DNS/UDP/未来 Chromium 通道及生产 Docker 网络均未完成独立审计；不可信书源可写入的公网部署仍应配置主机出口防火墙拦截云元数据、loopback、RFC1918 和 IPv6 ULA，并在真实部署网络验证。

### 2026-09-25 `sourceRegex` 与 HTML 字符集增量

本节覆盖上方较早快照中“仍不支持 `sourceRegex` 和非 UTF-8 `encode`”的结论：本地 Chromium 现在会先执行网络策略，再用完整正则匹配请求 URL；命中时捕获该资源 URL、终止该资源下载，并以 `StrResponse(原页面 URL, 命中资源 URL)` 返回。合成 Chrome 测试通过并确认目标夹具服务器没有收到被嗅探资源。对直接提供的 HTML，`encode` 现支持 JVM 可识别字符集，并通过指定字符集往返模拟 legacy `loadDataWithBaseURL` 的可表示字符与替换行为；GBK 中文与未知字符集错误测试均通过。HTTP 页面导航仍由 HTTP 头/HTML 元信息决定编码，不把 `encode` 错用于 HTTP 响应解码。

行为参考为[近似 legado Android `BackstageWebView` 实现](https://gitea.yamby.cn/yusheng/QieKan-3.0/src/commit/45ffb0ef213421373ad539e15880f4e2288f529e/app/src/main/java/io/legado/app/help/http/BackstageWebView.kt)：它对资源 URL 使用整串正则匹配，并把匹配的资源 URL 作为响应；这不是 `reader-pro-3.2.14.jar` 等版本证明。恢复版的 `sourceRegex`/字符集语义仍需与原 JAR 和真实远程 `/render.html` 服务做受控黑盒差分；期间保留远程 renderer 回退。此增量全量本机 Gradle + 已安装 Chrome 回归为 26 套件、56 用例、0 失败/错误/跳过，不能替代托管镜像验收或真实书源兼容测试。

### Camoufox 候选接入快照（2026-09-26，容器尚未验收）

本轮源码已加入 Reader 管理的 Python/Camoufox renderer、每次渲染独立浏览器上下文、现有 SSRF 出口代理、Cookie 用户命名空间回写、进程超时/回收以及非 HTTP 资源协议拦截。镜像默认配置指向 Camoufox；Python 包固定为 `0.5.6`、Playwright 固定为 `1.62.0`，浏览器版本锁定 `v152.0.4-beta.30`，安装脚本在解包前校验 GitHub release asset 的大小与 SHA-256。**已成功重建**：本机 Java/Kotlin 全量测试与 Vue 3 JAR 构建通过；**已在隔离 Windows 运行时验证**：真实 Camoufox GET/POST、脚本子资源、Cookie 回写和 `sourceRegex` 捕获通过。**尚未验证**：GitHub 托管 runner 的 Linux 双架构容器构建、镜像内合成及真实书源旅程、生产网络隔离、并发资源预算、ARM64 和部署回滚。旧 Chromium runner 与本机 Windows 测试均不能替代这些验证；当前生产版本也尚未包含此候选。

## 后续候选项（尚未承诺）

- 在不改变旧数据语义的前提下改善 JSON 存储性能；任何 SQLite 迁移都必须具备备份、校验和可回滚路径，且不能先于兼容基线验收。
- 扩展书源规则和本地书格式时，优先在现有 Kotlin/Java 工程中做增量实现与回归测试，避免再次引入全量语言重写。
- Vue 3 界面的工作范围和回退门槛以上述近期主线为准；不从 Rust 前端推断 Java/Kotlin 后端已经实现同名接口。

## 验证用语

维护记录应明确区分“已从 JAR 验证”“已从近似源码推断”“已成功重建”和“尚未验证”。共同失败的第三方书源不应记为恢复版回归；仅有 HTTP 建连的 SSE 不应记为完整成功；本机通过不应写成生产已部署。
