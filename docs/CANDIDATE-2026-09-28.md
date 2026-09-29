# Java/Kotlin + Vue 3 单容器候选验收记录（2026-09-28）

此记录跟踪候选分支 `ci/full-reader-20260926`，确切功能代码以该分支 Git HEAD 为准，不是正式发布说明。`legacy` 默认分支、`read.medwarp.cn` 生产容器和 `storage/data` 均未因该候选改变；原始 `reader-pro-3.2.14.jar` 继续保留作只读对照。下列本机 JAR 散列对应较早的 `0fd3a082` 验证点，不能冒充当前提交的制品散列。

## 已成功重建与本机验证

- JDK 11、Gradle 6.1.1 执行 `./gradlew -PreaderWebUi=vue3 clean test bootJar --no-daemon` 成功。JUnit 结果为 82 项、0 失败、0 错误、12 跳过；跳过项包含 3 项本机缺少 Camoufox 的真实浏览器测试与 9 项本机缺少 Chromium 的测试，不能写成 82 项全部执行成功。
- `web-vue3` 的 Node 测试 133/133 通过，`vue-tsc --noEmit` 与 Vite 生产构建通过；Python worker 协议测试 7/7 通过。
- 本机产物 `build/libs/reader-4.0.7.jar` 为 285,614,283 字节，SHA-256 为 `53CB873590D895CA9AF286CD85B20C910AAB823CF15ABB54762F97D731B3D36F`。JAR 内已核对 `web-vue3/index.html`、`web-vue3/sw.js`、`camoufox/worker.py`、Playwright Java 与 driver bundle。散列仅标识此次本机构建，不等同于 GitHub 发布制品，也不证明字节级可重复构建。
- Cookie bridge 已改用带 Domain、Path、Secure、HttpOnly、过期时间的结构化存储并按用户命名空间隔离；普通 HTTP 响应、历史平面 Cookie 迁移、Domain 删除、HTTP 不安全来源覆盖 Secure Cookie 和单次显式请求头均有定向测试。worker 正文/协议输出分别限制为 4/8 MiB，父进程 stdout 限制为 8 MiB；超限后恢复用例已通过。后续提交增加了 JavaScript 删除快照、公共后缀和精确 Path 回归；这些改动仍以 hosted Linux 真实 Camoufox 结果为最终门禁。
- Vue 3 首次安装 Service Worker 不再在登录输入期间强制重载；更新改为用户确认后激活。本机独立浏览器此前验证了注册后自动登录、书架、阅读、搜索、书源、分组、替换规则、文件和备份等基础旅程，但这不是生产或正式镜像内验收。复核发现旧 `Vue3PreviewLoginTest` 仅覆盖注册后自动登录，不足以证明已有账号能从可见表单重新登录；`0dc24b57` 已补入隔离账号退出后重新登录与书架请求断言，须以 hosted Chromium 执行且 `skipped=0` 为准。
- 本机隔离构建增加可选 `reader.server.bindAddress`；默认 `0.0.0.0` 保持旧部署行为，本机以 `127.0.0.1` 启动后，`netstat` 确认仅回环监听、`/reader3/getSystemInfo` 返回 HTTP 200。JDK 11/Gradle 6.1.1 离线 `test bootJar` 成功：84 项、0 失败、0 错误、13 跳过；产物 SHA-256 `A6B84FA76C628EA46E40DB5C7BCB8B115F2DE245757506C86AD5DF846397AE3E`。全新临时工作目录只生成合成账号，测试进程与该临时目录已清理。Chrome 中 Vue 3 登录表单和中文显示正常，但人工浏览器提交被自动化安全审查拦下；此轮不把它记作新的 UI 登录通过，继续以已通过的隔离 Chromium 旅程为既有证据。
- **已从原始 JAR 验证**：`BOOT-INF/lib/jackson-module-kotlin-2.13.5.jar`。因此把恢复版直接依赖从浮动 `2.13.+` 固定为 `2.13.5`；这消除了本机首次构建时对 Maven 版本元数据的在线查询。固定后离线 `test bootJar` 再次成功，84 项、0 失败、0 错误、13 跳过，JAR SHA-256 `A1F6E1BC8FD094232C65A3CA6E80207EA5595744DF8B40F68249074D4156AB96`。这是单次产物散列，不宣称完整字节级可重复构建。

## GitHub 托管 runner 验收

最初的 `0fd3a082` 三道门禁中，[Java/Kotlin CI](https://github.com/warpdotsys/reader-dev/actions/runs/36363337667) 和 [Vue 3 浏览器旅程](https://github.com/warpdotsys/reader-dev/actions/runs/36363336291) 已通过；[单容器浏览器集成](https://github.com/warpdotsys/reader-dev/actions/runs/36363341230) 在 Linux Camoufox Cookie 回归失败。第二轮 `e9e42ad6` 的 [Java/Kotlin CI](https://github.com/warpdotsys/reader-dev/actions/runs/36364562161) 与 [Vue 3 旅程](https://github.com/warpdotsys/reader-dev/actions/runs/36364565231) 通过，但 [单容器集成](https://github.com/warpdotsys/reader-dev/actions/runs/36364566276) 在同类 Cookie 用例仍失败。两次失败都明确阻止发布，不能挑绿灯声称整体成功。

`13664541` 的 [单容器浏览器集成](https://github.com/warpdotsys/reader-dev/actions/runs/36364945349) 在测试包装后的纯文本响应断言失败；保存的 JUnit 报告证明是 Firefox 将 `text/plain` 包在 HTML 中，未证明 Cookie 逻辑出错。`5c077810` 的 [单容器浏览器集成](https://github.com/warpdotsys/reader-dev/actions/runs/36365448082) 已通过真实 Camoufox GET/POST、脚本、Cookie、精确 Path 子资源和 sourceRegex 合约，但镜像构建末尾因非 root 用户无法删除 root 所有的临时安装文件而失败。该权限问题在 `cf7ffa2c` 修复。

当前 `cf7ffa2c` 的 [Java/Kotlin CI](https://github.com/warpdotsys/reader-dev/actions/runs/36366116761)、[Vue 3 浏览器旅程](https://github.com/warpdotsys/reader-dev/actions/runs/36366123499) 与 [单容器浏览器集成](https://github.com/warpdotsys/reader-dev/actions/runs/36366072758) 均在 GitHub 托管 runner 上通过。第三道门禁包括真实 Camoufox 合约、完整镜像构建，以及受限容器内的 Reader 接口和 WebView 合成书源冒烟。

三个 workflow 的职责：

- Java/Kotlin CI：前后端测试与 JAR。
- Vue 3 浏览器旅程：隔离账号、真实 Chromium、构建后入口。
- 单容器浏览器集成：固定 Camoufox、Linux renderer 合约、完整镜像与合成书源烟测；容器设置 2 GiB 内存、3 GiB memory+swap、256 PIDs 与 2 CPU 的上限。

三道门禁全绿是候选构建和合成流程验收，不是原版兼容性证明。[一条公开真实书源的原 JAR／恢复版静态目录差分](REAL-SOURCE-DIFF-2026-09-28.md)已通过；`6427f09a` 的 [GitHub 单容器作业](https://github.com/warpdotsys/reader-dev/actions/runs/36367557675)还以手动实站选项验证了镜像内 Camoufox 能从同一公开站点解析 10 本书。但仍缺真实 WebView 三方差分和正式镜像/生产负载验收，不得据此创建正式标签、推送发布镜像或切换生产。

`7710518d` 的设置页修正已在 [Java/Kotlin CI](https://github.com/warpdotsys/reader-dev/actions/runs/36416844091)、[Vue 3 Chromium 浏览器旅程](https://github.com/warpdotsys/reader-dev/actions/runs/36416847605)和 [完整单容器镜像集成](https://github.com/warpdotsys/reader-dev/actions/runs/36416847244) 三道 GitHub 托管 runner 门禁通过。浏览器旅程使用隔离账号，不是生产账号。三道作业未在本次启用公开真实书源选项；前一轮真实书源结果仍需按其原提交范围解释。

本轮 `f3e95b23` 的 [Java/Kotlin CI](https://github.com/warpdotsys/reader-dev/actions/runs/36432696623)与[完整单容器镜像构建和容器内冒烟](https://github.com/warpdotsys/reader-dev/actions/runs/36432691482)通过；`b472589c` 的 [Vue 3 Chromium 浏览器旅程](https://github.com/warpdotsys/reader-dev/actions/runs/36433301875)通过。两提交之间只修正了该 Vue 3 workflow 的 Linux 回环监听文本断言，产品源码与锁定依赖未变。此前 [5192ef51 的 Vue 3 作业](https://github.com/warpdotsys/reader-dev/actions/runs/36431567550)和 [f3e95b23 的 Vue 3 作业](https://github.com/warpdotsys/reader-dev/actions/runs/36432659453)均在浏览器旅程开始前被此断言挡住：Linux `ss` 实际显示 `[::ffff:127.0.0.1]:18895`，不是普通 `127.0.0.1:18895`；修正后隔离账号旅程已跑完。`browser-image.yml` 中的原 JAR 来源探针和旧远程 WebView 探针是可选作业，本轮默认均跳过，不能算作三方差分通过。

`1e91d94f` 修正 Camoufox 对对象、数组、数字型 JavaScript 返回值的编码，使其按旧参考镜像实测的紧凑 JSON 格式返回。其 [Java/Kotlin CI](https://github.com/warpdotsys/reader-dev/actions/runs/36438478628)通过；含同一产品源码的 `61d5b88c` [完整镜像作业](https://github.com/warpdotsys/reader-dev/actions/runs/36439102342)通过 JAR 构建、真实 Camoufox 合约、单容器镜像构建及受限容器内 API/WebView 合成书源烟测。该作业 JAR SHA-256 为 `BCE9266B1F667DCA5A0C7C73ED2AE8A08AF45653E58F7ECD9E4D0614BDEC5BB9`；这是此 runner 的产物标识，不证明跨环境字节级可重复构建。随后提交只改了参考探针及文档，不改变产品逻辑。`ebda1164` 的 [Vue 3 预览构建与隔离 Chromium 旅程](https://github.com/warpdotsys/reader-dev/actions/runs/36440229851)两个作业均通过；该旅程验证候选登录、书架、阅读、搜索、书源等既定测试场景，不等于生产 UI 验收。

`51a91cc9` 的 [2026-09-29 完整镜像作业](https://github.com/warpdotsys/reader-dev/actions/runs/36508005109)在 GitHub 托管 runner 成功：真实 Camoufox 合约、单镜像构建、2 GiB 内存／256 PID／2 CPU 上限中的合成烟测均通过；后者在四个独立账号并发搜索下分别得到一本书。该次 `docker image inspect .Size` 为 **5,611,994,503 字节**（约 5.23 GiB，未压缩镜像大小，不是传输量或宿主机实际占用）；容器 cgroup `memory.peak` 为 **778,379,264 字节**（约 742 MiB），`pids.peak` 为 **176**。这仅是短时合成负载的峰值，不代表真实站点或长时间运行的容量上限。其后的 `81f14c19` 将浏览器 Cookie 所有权改为显式能力声明，仍需对该提交单独完成 hosted 门禁；上述数值不可自动外推到新提交。

## 已知问题与尚未验证

- 当前单镜像体积偏大：`51a91cc9` 作业测得未压缩镜像约 5.23 GiB。该值不代表实际拉取流量；目标服务器的可用磁盘、镜像拉取耗时和升级时新旧镜像共存空间尚未验收。发布前必须核对这些条件；后续如更换更小的基础镜像，仍须重跑真实 Camoufox、字体和完整镜像烟测，不能仅凭体积下降验收。
- 真实需登录书源仍缺少“原始 JAR／现有远程 WebView／内置 Camoufox”同条件三方差分；已经通过的公开静态书源、镜像内公开 WebView 和合成书源都不能替代这项兼容验收。2026-09-28 的只读核查确认两条无登录凭据的真实 WebView 候选站点可达，但尚未执行三方请求；旧 Reader 所引用的远程 WebView 服务在部署主机上不存在运行中或已停止的容器，也无法从旧容器解析，因此不能把旧远程服务误记为已测对照。浏览器内 JavaScript 主动删除既有 Cookie、复杂 Set-Cookie 日期/引号与跨站 SameSite 行为仍缺少完整端到端覆盖；JavaScript 删除目前只有快照协议单测。
- 公共镜像站的 `hectorqin/reader:3.2.14` 中 JAR 已在 [托管 runner](https://github.com/warpdotsys/reader-dev/actions/runs/36418763589)与本地原件做大小和 SHA-256 核对，**不一致**；不得把它冒充原 JAR 来完成三方差分。官方 Docker Hub 的同名 Reader 标签当前返回 404；旧远程 WebView 3.2.0 镜像可获取，但其是否为原生产实例仍未证实。证据见[来源核验](ORIGINAL-JAR-PROVENANCE-2026-09-28.md)。
- 固定摘要的旧远程 WebView 参考镜像在隔离 [托管 runner](https://github.com/warpdotsys/reader-dev/actions/runs/36437640038)上通过合成 GET、POST、公开目录以及脚本返回字符串、对象、数组和数字的探针。首次脚本探针因只有页面副作用而没有返回值，走了旧实现的空值重试分支；该超时不是已证实的服务故障。参考响应摘要、Cookie 与证据边界见[旧服务参考探针](ARCHIVED-REMOTE-WEBVIEW-2026-09-28.md)。Camoufox 曾把对象/数组调用 `toString()`，与已实测的参考 JSON 格式不同；候选源码已针对这一差异修改并通过上述真实浏览器和完整镜像门禁。不能把参考结果写成原 JAR 三方兼容通过。
- 同一固定参考镜像的 `sourceRegex` 合成样本已从页面实际请求资源，但监听器未返回；源码使用 `request().url.match` 而非 Playwright API 的 `request().url().match`。这是[本镜像的已知缺陷](https://github.com/warpdotsys/reader-dev/actions/runs/36439381876)，在[预期缺陷探针](https://github.com/warpdotsys/reader-dev/actions/runs/36439959686)中复现并明确标记，不作为候选浏览器必须复现的行为，也不证明此前生产远程服务具有同样缺陷。
- [远程回退合成差分](REMOTE-WEBVIEW-FALLBACK-DIFF-2026-09-29.md)发现并修复一处 Cookie 回放回归：为内置浏览器跳过 Cookie 请求头的条件曾误覆盖远程渲染器。修复后的本机 JAR 与原始 JAR 均把书源 `webJs` 传为 `js_source`；恢复版的结构化 Cookie 回放仍是有意的兼容性差异。此增量尚待托管 runner 重测，不等于原生产 WebView 的三方验收。
- 2026-09-28 与 2026-09-29 使用真实 Chrome 表单对当前线上原版完成登录；后一次退出旧会话再登录，读取到 169 本书；见[线上登录核验](PRODUCTION-CHROME-LOGIN-2026-09-28.md)。这仅证实现部署的 Chrome 路径在这两次测试成功，Edge 与本候选的线上 UI 登录仍未验证。
- 本机缺少完整 Linux Camoufox 环境；hosted image 作业已确认真实浏览器与受限容器启动、合成请求和资源预算，但真实书源超时、生产负载、并发资源预算和 ARM64 尚未验收。
- GitHub 托管 runner 当前将旧 Node.js 20 action 强制运行于 Node.js 24，并提示 `setup-java@v4` 维护期结束。现有门禁通过，但后续应升级并锁定新版本 action，避免未来平台迁移导致 CI 失效。
- 单页正文 UTF-8 超过 4 MiB 会显式失败；这是一条安全/内存预算限制，可能影响超大章节或页面，需要用真实书源样本验证后决定是否调整。不能静默截断正文。
- Vue 3 的基础旅程通过不代表与旧前端所有设置、书源规则和异常提示逐项一致。当前生产仍是原界面，线上登录只确认过一次成功进入书架并刷新后保持会话，未证明所有用户故障消失。
- 2026-09-28 本地设置页视觉核查发现关于页沿用 Rust 版本履历、使用并不存在的后端统计字段、首次使用的云端偏好提示误报失败、OPDS 令牌明文显示；候选代码已修正，136 项 Node 测试、类型检查、Vite 构建和上述三道 hosted workflow 通过。早期只测生产登录 API 不能证明页面可用；后来已单独完成[生产 Chrome 表单登录](PRODUCTION-CHROME-LOGIN-2026-09-28.md)，但这仍不是恢复候选的生产验收。

## 发布与回滚边界

正式版本必须先等 hosted 三道门禁全绿，并补齐真实书源差分、已知问题和版本同步。`release.yml` 的稳定标签会发布 GHCR/Docker Hub 镜像并触发生产部署，因此验证前不得创建该标签。部署前需保留旧镜像与 `storage/data` 一致性快照，部署后验证公网登录、书架、书源、阅读和下载；任一检查失败须回到旧镜像与对应数据快照。只有现有远程 WebView 服务可用时，才可将 `READER_APP_WEBVIEWRENDERER` 回切到 `remote`，不能把切换配置当成数据回滚。
