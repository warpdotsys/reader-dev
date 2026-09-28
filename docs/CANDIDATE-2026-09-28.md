# Java/Kotlin + Vue 3 单容器候选验收记录（2026-09-28）

此记录跟踪候选分支 `ci/full-reader-20260926`，确切功能代码以该分支 Git HEAD 为准，不是正式发布说明。`legacy` 默认分支、`read.medwarp.cn` 生产容器和 `storage/data` 均未因该候选改变；原始 `reader-pro-3.2.14.jar` 继续保留作只读对照。下列本机 JAR 散列对应较早的 `0fd3a082` 验证点，不能冒充当前提交的制品散列。

## 已成功重建与本机验证

- JDK 11、Gradle 6.1.1 执行 `./gradlew -PreaderWebUi=vue3 clean test bootJar --no-daemon` 成功。JUnit 结果为 82 项、0 失败、0 错误、12 跳过；跳过项包含 3 项本机缺少 Camoufox 的真实浏览器测试与 9 项本机缺少 Chromium 的测试，不能写成 82 项全部执行成功。
- `web-vue3` 的 Node 测试 133/133 通过，`vue-tsc --noEmit` 与 Vite 生产构建通过；Python worker 协议测试 7/7 通过。
- 本机产物 `build/libs/reader-4.0.7.jar` 为 285,614,283 字节，SHA-256 为 `53CB873590D895CA9AF286CD85B20C910AAB823CF15ABB54762F97D731B3D36F`。JAR 内已核对 `web-vue3/index.html`、`web-vue3/sw.js`、`camoufox/worker.py`、Playwright Java 与 driver bundle。散列仅标识此次本机构建，不等同于 GitHub 发布制品，也不证明字节级可重复构建。
- Cookie bridge 已改用带 Domain、Path、Secure、HttpOnly、过期时间的结构化存储并按用户命名空间隔离；普通 HTTP 响应、历史平面 Cookie 迁移、Domain 删除、HTTP 不安全来源覆盖 Secure Cookie 和单次显式请求头均有定向测试。worker 正文/协议输出分别限制为 4/8 MiB，父进程 stdout 限制为 8 MiB；超限后恢复用例已通过。后续提交增加了 JavaScript 删除快照、公共后缀和精确 Path 回归；这些改动仍以 hosted Linux 真实 Camoufox 结果为最终门禁。
- Vue 3 首次安装 Service Worker 不再在登录输入期间强制重载；更新改为用户确认后激活。本机独立浏览器此前验证了登录、书架、阅读、搜索、书源、分组、替换规则、文件和备份等基础旅程，但这不是生产或正式镜像内验收。

## GitHub 托管 runner 验收

最初的 `0fd3a082` 三道门禁中，[Java/Kotlin CI](https://github.com/warpdotsys/reader-dev/actions/runs/36363337667) 和 [Vue 3 浏览器旅程](https://github.com/warpdotsys/reader-dev/actions/runs/36363336291) 已通过；[单容器浏览器集成](https://github.com/warpdotsys/reader-dev/actions/runs/36363341230) 在 Linux Camoufox Cookie 回归失败。第二轮 `e9e42ad6` 的 [Java/Kotlin CI](https://github.com/warpdotsys/reader-dev/actions/runs/36364562161) 与 [Vue 3 旅程](https://github.com/warpdotsys/reader-dev/actions/runs/36364565231) 通过，但 [单容器集成](https://github.com/warpdotsys/reader-dev/actions/runs/36364566276) 在同类 Cookie 用例仍失败。两次失败都明确阻止发布，不能挑绿灯声称整体成功。

`13664541` 的 [单容器浏览器集成](https://github.com/warpdotsys/reader-dev/actions/runs/36364945349) 在测试包装后的纯文本响应断言失败；保存的 JUnit 报告证明是 Firefox 将 `text/plain` 包在 HTML 中，未证明 Cookie 逻辑出错。`5c077810` 的 [单容器浏览器集成](https://github.com/warpdotsys/reader-dev/actions/runs/36365448082) 已通过真实 Camoufox GET/POST、脚本、Cookie、精确 Path 子资源和 sourceRegex 合约，但镜像构建末尾因非 root 用户无法删除 root 所有的临时安装文件而失败。该权限问题在 `cf7ffa2c` 修复。

当前 `cf7ffa2c` 的 [Java/Kotlin CI](https://github.com/warpdotsys/reader-dev/actions/runs/36366116761)、[Vue 3 浏览器旅程](https://github.com/warpdotsys/reader-dev/actions/runs/36366123499) 与 [单容器浏览器集成](https://github.com/warpdotsys/reader-dev/actions/runs/36366072758) 均在 GitHub 托管 runner 上通过。第三道门禁包括真实 Camoufox 合约、完整镜像构建，以及受限容器内的 Reader 接口和 WebView 合成书源冒烟。

三个 workflow 的职责：

- Java/Kotlin CI：前后端测试与 JAR。
- Vue 3 浏览器旅程：隔离账号、真实 Chromium、构建后入口。
- 单容器浏览器集成：固定 Camoufox、Linux renderer 合约、完整镜像与合成书源烟测；容器设置 2 GiB 内存、3 GiB memory+swap、256 PIDs 与 2 CPU 的上限。

三道门禁全绿是候选构建和合成流程验收，不是原版兼容性证明。[一条公开真实书源的原 JAR／恢复版静态目录差分](REAL-SOURCE-DIFF-2026-09-28.md)已通过；`6427f09a` 的 [GitHub 单容器作业](https://github.com/warpdotsys/reader-dev/actions/runs/36367557675)还以手动实站选项验证了镜像内 Camoufox 能从同一公开站点解析 10 本书。但仍缺真实 WebView 三方差分和正式镜像/生产负载验收，不得据此创建正式标签、推送发布镜像或切换生产。

## 已知问题与尚未验证

- 真实需登录书源仍缺少“原始 JAR／现有远程 WebView／内置 Camoufox”同条件三方差分；已经通过的公开静态书源、镜像内公开 WebView 和合成书源都不能替代这项兼容验收。2026-09-28 的只读核查确认两条无登录凭据的真实 WebView 候选站点可达，但尚未执行三方请求；旧 Reader 所引用的远程 WebView 服务在部署主机上不存在运行中或已停止的容器，也无法从旧容器解析，因此不能把旧远程服务误记为已测对照。浏览器内 JavaScript 主动删除既有 Cookie、复杂 Set-Cookie 日期/引号与跨站 SameSite 行为仍缺少完整端到端覆盖；JavaScript 删除目前只有快照协议单测。
- 本机缺少完整 Linux Camoufox 环境；hosted image 作业已确认真实浏览器与受限容器启动、合成请求和资源预算，但真实书源超时、生产负载、并发资源预算和 ARM64 尚未验收。
- GitHub 托管 runner 当前将旧 Node.js 20 action 强制运行于 Node.js 24，并提示 `setup-java@v4` 维护期结束。现有门禁通过，但后续应升级并锁定新版本 action，避免未来平台迁移导致 CI 失效。
- 单页正文 UTF-8 超过 4 MiB 会显式失败；这是一条安全/内存预算限制，可能影响超大章节或页面，需要用真实书源样本验证后决定是否调整。不能静默截断正文。
- Vue 3 的基础旅程通过不代表与旧前端所有设置、书源规则和异常提示逐项一致。当前生产仍是原界面，线上登录只确认过一次成功进入书架并刷新后保持会话，未证明所有用户故障消失。
- 2026-09-28 本地设置页视觉核查发现关于页沿用 Rust 版本履历、使用并不存在的后端统计字段、首次使用的云端偏好提示误报失败、OPDS 令牌明文显示；候选代码已修正，136 项 Node 测试、类型检查和 Vite 构建通过，但这些修正尚未由新提交的 hosted browser workflow 验收。生产旧界面的新登录尝试在提交前因浏览器连接中断，不能据此宣布登录正常或故障已复现。

## 发布与回滚边界

正式版本必须先等 hosted 三道门禁全绿，并补齐真实书源差分、已知问题和版本同步。`release.yml` 的稳定标签会发布 GHCR/Docker Hub 镜像并触发生产部署，因此验证前不得创建该标签。部署前需保留旧镜像与 `storage/data` 一致性快照，部署后验证公网登录、书架、书源、阅读和下载；任一检查失败须回到旧镜像与对应数据快照。只有现有远程 WebView 服务可用时，才可将 `READER_APP_WEBVIEWRENDERER` 回切到 `remote`，不能把切换配置当成数据回滚。
