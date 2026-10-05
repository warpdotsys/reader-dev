# 默认 Camoufox 导航契约门禁

更新日期：2026-10-05。基于 `3315d9cf` 新增回归、保留两次真实失败后修复可读 worker；浏览器二进制/依赖不变。Chromium 内容快照修复不能替代默认引擎实测。

## 本增量

在现有真实 Camoufox 端到端类追加纯生成导航：初始 POST 后的 DOMContentLoaded 连续触发 13 个中间文档以及最终页面。连续三套新生成命名空间必须读到最终标记、不能混入中间页面，服务端计数必须只有三个初始请求/三个 POST。起始响应的生成 HttpOnly Cookie 必须在最终状态实际保留，不重发表单、不使用网站账号或真实正文。

回归/诊断阶段固定 Camoufox `152.0.4-beta.30` 和既有依赖，未修改渲染器/worker；没有先假定 Firefox 存在相同竞态，也没有把 Chrome 结果当其结论。后来确认的真实失败与 worker 修复另记于下节。

同时补齐托管 Camoufox 门禁：JUnit 必须精确 13 项、0 跳过/失败/错误且包含这个真实导航用例；XML 缺失从警告改为失败。原先只有 Gradle 成功不足以排除整类环境门控跳过。失败 XML 仍以 `always()` 保留，不隐藏失败来发版。

## 已验证与尚未验证

- 本机 Kotlin 测试编译成功，但 Camoufox Python/二进制没有配置，**13 项全部门控跳过**；不是真实默认引擎成功或失败。原 XML SHA-256 `03f198048a9adbc13a2fbf177be7a20a3fa04b9a8df5465ffe538573135975f5` 已冻结。
- 本次门禁的 actionlint 与 47 项发布链路测试通过，未运行 shellcheck/pyflakes。新用例源码 SHA-256 `ed3f77cb53dd6d685f16f27ce671f22c035338f4f85112d8e12e54929bd447d2`。
- 前一 `3315d9cf` Browser 工作流 37294389490 已完成：下载的四套 XML 共 35 项，0 跳过/失败/错误，包含真实 Chromium 导航与旧 12 项 Camoufox；另两份完整镜像 JSON 已核对。**旧 12 项未包含本新增导航，不得用于证明它。**
- **已从真实托管运行验证失败**：源码 `26255c06732dd05fb519b7a9943a96490bc53266` 的 [Browser 工作流 37296514243](https://github.com/warpdotsys/reader-dev/actions/runs/37296514243) 中运行时安装成功，实际 XML 为 13 项、1 失败、0 错误、0 跳过。旧 12 项通过，新增导航用例在第一个最终文档断言失败（3.016 秒），不是 Chromium 的内容读取异常。失败制品 ID `11339482304`，XML SHA-256 `34ffe21a78340bb317b2e2567f1bc9179ad3d155d8988cbc10f033b1b61770e3` 已冻结；此时还不能确认返回的是中间页面还是空值，也不能宣称后续 Cookie/三次 POST 断言已通过。
- 诊断增量仅添加生成页面的长度、标记存在性、最大跳转步数、最终页访问次数和初始 POST 计数，不输出 HTML、Cookie 或正文，不弱化最终页断言。本机测试编译成功但仍 13 项全跳过；该轮 XML SHA-256 `71c425756c14b923d0c7c0738ac672d0cb74248351cf8e5d9c9d5039a28d7079`。诊断源码 SHA-256 `7ceb5311ada67fabfa43439ff04097000bc30b7c67aa18e7b440b945e96dccd9`；当时生产 worker/renderer 未改，随后真实诊断失败见下节。
- 同一 `26255c0` 的 Java 与 Native 工作流成功（37296514242/37296514255，后者本轮未逐制品重新验收）；Vue 工作流 37296514253 失败。下载的跨标签 XML 显示甲退出后乙 token 的 `getUserInfo` 返回 `isSuccess=false`（调用行 242），不是共享 Cookie 断言行 243。先前账号归属/重连断言通过不能覆盖这个失败；原因待查，不重启旧作业来掩盖失败。

[结构化失败与诊断边界](evidence/camoufox-navigation-2026-10-05.json)。候选不得因其他工作流成功而发版。

## 真实诊断与修复候选

源码 `6d9379706ac0c1e8861f82b77054614f5cc52ca4` 的 [托管 Browser 运行 37298826053](https://github.com/warpdotsys/reader-dev/actions/runs/37298826053) 再次失败：实际 13 项/1 失败/0 跳过，新增用例 1.795 秒。生成诊断为 `round=0 snapshotChars=205 intermediate=true maxStep=1 finalVisits=0 starts=1 posts=1`，证明 worker 在最终页尚未访问时已返回中间 HTML，而不是空值、重复 POST 或 Chromium 抛出的快照异常。失败制品 `11339929899`（2178 字节），XML SHA-256 `e4155c2753b0b40516d0f63e121d41c00dfc5e9c01dbed915b7477d3dda6c79b` 已另目录冻结，未覆盖第一次失败。

修复仅用于无 `sourceRegex`/无显式源脚本的 HTML 快照路径：在首次导航之前观察主 frame 的导航请求、请求结束与文档提交；等待主文档无待完成导航并安静 200 毫秒，核对 DOMContentLoaded 与快照前后代数一致。事件循环由浏览器等待推进，不用 `time.sleep` 隐藏新导航；不重发 goto/POST，不重执行页面或书源脚本，不以整个网络 `networkidle` 等待 SSE。一个单调时钟快照预算覆盖整条跳转链，协议/正文限额、代理、安全拒绝和父进程看门狗保持。未知内容读取/关闭页面/传输异常仍失败，不吞掉来返回旧文档。

根据 [Playwright 的加载状态契约](https://playwright.dev/python/docs/api/class-page#page-wait-for-load-state)，当前文档已达到状态时等待会立即完成；因此单次 DOMContentLoaded 不能证明整条客户端导航结束。观察使用官方 [frame 导航事件](https://playwright.dev/python/docs/api/class-page#page-on-framenavigated) 和[导航请求标记](https://playwright.dev/python/docs/api/class-request#request-is-navigation-request)，未改网站指纹、网络策略或私网允许条件。

**已本机单元验证**：新增 9 项确定性生成事件测试覆盖待完成主请求、快照/加载状态中的新提交、子 frame/非导航流不阻塞、无限提交的单预算、网络拒绝、未知异常、UTF-8 限额和非法预算；Python 共 97 项，96 执行、1 环境跳过，无失败。事件替身不是实际浏览器证明。真实 13 项与完整镜像/双架构必须等待修复候选自己的托管报告。

**已成功重建候选**：本机完整 Java/Kotlin 测试为 41 套/147 项，实际执行 134、无失败/错误；13 项默认 Camoufox 仍因运行时缺失跳过，不能计为浏览器通过。Vue 3 JAR SHA-256 `8a8b6bf2d6d8c7e403b6c336a364f686916f3e207252d499f158a3d8bfeca96e`；81 个包内 UI 文件与 dist 逐字节一致，新 worker 确实打入包且与源码一致。完整 XML 冻结在 `build/camoufox-navigation-20261005-a/candidate-local-full/`。不以本机成功替代托管实际浏览器。

已知限制：200 毫秒安静窗口不是任意未来定时脚本的最终页保证；原有父进程预算不是每个浏览器同步 API 的操作系统硬抢占。没有新验证无限真实页面导航、原 JAR/远程引擎对该用例的三方差分、真实起点认证/精确搜索/正文或生产。

不触及用户脏 reports、原始 JAR、已有三个工程、系统 hosts、用户浏览器会话或生产；不导入已清除凭据、不处理验证码、不购买、不请求章节正文。起点精确书名搜索/真实认证三方和完整产品验收仍未完成。没有合并默认分支、正式发版或生产部署。

回滚在独立干净工作树撤回后续 worker 改动即可恢复 `6d937970` 的运行时；建议保留失败回归与门禁，不隐藏已知问题。若撤回测试/门禁则会再次失去默认引擎导航覆盖和禁止跳过的强校验。不得对用户工作区 reset/checkout 覆盖。会话存储窗口是另一个已复现缺陷，见[存储读取一致性](STORAGE-READ-GAP-2026-10-05.md)，不能把它与默认浏览器问题合并成一个根因。
