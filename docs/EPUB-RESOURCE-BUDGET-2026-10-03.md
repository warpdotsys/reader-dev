# EPUB 原版排版：资源预算、取消与嵌套 CSS

日期：2026-10-03。仅记录 Vue 3 候选的增量验收，不代表正式发布、生产替换或整个项目完成。后端仍为 Java/Kotlin；没有改写 legacy 章节编号、HTTP 契约、用户命名空间或存储格式。

## 已成功重建与本机验证

原版排版的下载按实际流量计数，不能只相信 `Content-Length`。ZIP 先核对中央目录，再以小块核对本地条目和实际解压输出；重复归一化路径、越根路径、大小低报及不一致会明确失败，不返回被截断的正文或伪造空资源。生产路径使用专用 Worker，转移输入和输出；解压任务 15 秒超时即终止，切换普通模式或卸载页面会取消下载并终止任务，迟到结果不回写。

固定前端预算如下。限制仅作用于原版排版，不擅自修改原书；超限时显示具体原因，提供“切换普通阅读”与重试入口。

| 项目 | 上限 |
| --- | --- |
| EPUB 压缩下载 | 64 MiB |
| ZIP 实际展开总量 | 256 MiB |
| 单项二进制资源 | 32 MiB |
| 单项 XML/OPF/NCX/XHTML/HTML/CSS | 4 MiB |
| ZIP 条目 | 8192 |
| 单入口 CSS 递归深度/引用次数 | 16 / 128 |
| 重写后的 CSS blob 总量 | 32 MiB |
| 整本书 blob URL | 16384 |

CSS 以各张样式表自己的目录解析递归 `@import` 和图片/字体 URL，保留 media、layer、supports 条件；循环只忽略递归边，不丢掉表内规则。链接样式通过 CSS blob 加载，行内样式单独重写，不把 CSS 拼接进 HTML。外部协议及无法定位的资源不会发出书外请求。保留 iframe 的 CSP、脚本清理与 `sandbox="allow-same-origin"`。条件语法参考 [W3C CSS Cascade 5](https://www.w3.org/TR/css-cascade-5/#at-import)；不是完整 CSS 语法解析器或原 JAR 行为等价证明。

新增单元覆盖 18 项，前端全量 **173 项、0 跳过/失败**；Python **45 项通过**，发布结构检查与 **5 项负向检查通过**。类型检查、Vite 构建和独立 bootJar 成功，实际包含 `epubArchive.worker-DofPich5.js`。最后受测 JAR SHA-256 为 `b4f56c38d2851677483be949f98631f2b2b5bf4bf10b2b7e338f837b4e9835f6`。

## 有效失败基线与修复后完整旅程

`Vue3PreviewLocalReadingTest` 一项 JUnit 包含六本生成样本，不是六项独立 JUnit：TXT、普通双 XHTML EPUB、NCX 同页锚点、EPUB3 nav 跨文件、嵌套 CSS、超限 CSS，最后重复普通 CSS 书验证取消下载。原有可控旧 POST 延迟断言继续执行，不能用新 CSS 通过掩盖进度竞争回归。

- **有效旧候选失败**：`74fbd5a5...084306`，57.353 秒。正确 `/reader/{bookUrl}` 阅读路由、原版模式、实际元素均存在；三层 import 的预期颜色 `rgb(17, 34, 51)` 未生效，实际 `rgb(0, 0, 0)`。失败 XML 和生成截图保留。
- **第一份新构建发现真实 UI 问题**：`d28e244c...514539`，57.610 秒。CSS 检查已通过；随后超限的重复悬浮提示拦截排版按钮，鼠标悬停导致提示持续不消失。保留失败记录，修复为页内错误与直接普通阅读入口，不用强制点击绕过遮挡。
- **最终新构建完整通过**：`b4f56c38...9835f6`，19.346 秒，1 项、0 跳过/失败/错误。实际检查嵌套颜色、粗体、循环规则的 margin/padding、书内背景 blob、行内背景与 12 px 图片、脚本清理及无书外请求；超限时无 iframe、无重复遮挡 toast，直接普通阅读按钮恢复正文；扣留实际下载后切换普通模式，Chromium 报告 `ERR_ABORTED`，无迟到 iframe 或错误。

CSS、超限提示和普通阅读恢复三张**生成样本**截图已目视核验，中文清晰、按钮可见。没有上传私人截图或正文。报告/产物摘要见[机器可读证据](evidence/epub-resource-budget-local-2026-10-03.json)。

更早三次诊断使用了错误的 `/reader?url=...` 路由，实际 404；它们保存在忽略的 `build` 目录，明确排除于产品失败基线。修正路由、按按钮实际标签选择模式以及为两本生成书使用不同标题，是测试夹具修复，不冒称业务修复。

## 唯一授权真实 EPUB 与原 JAR 的证据边界

仅《黎明之剑》获准读取，已存在的隔离副本保持只读。本轮只检查实际资源预算与解析：压缩 37,809,796 字节、1673 条目、实际展开 49,382,784 字节、最大资源 1,963,103 字节、1611 spine 项，约 1208 ms；新预算接受，原输入不变。此统计不是新 Worker 的全书界面验收，不证明所有字体/样式正确，也不公布书内文件名或正文。

原 JAR `b26fb476...030c8c` 和该授权副本 `bd8c6819...f3ced31` 的 SHA-256 复核不变。用户此前授权的服务器隔离差分已实际完成，[两本 NCX/nav 生成样本的原 JAR 与恢复 JAR 对照](EPUB3-NAV-ACCEPTANCE-2026-10-03.md#原-jar-同条件对照入口与当前状态)相同；那次恢复 JAR 为 `74fbd5a5...084306`，不能拼接为本轮新 JAR、原 UI 或复杂 CSS 同条件差分。

## 新源码的实际 GitHub 托管验收

功能提交 `c2892805cdd59d887aea89697122fce3cfb4e0d4` 三道门禁已全部成功；实际 runner 标签 `ubuntu-24.04`、名称 GitHub Actions，不是自托管：

- [Java/Kotlin 37120476610](https://github.com/warpdotsys/reader-dev/actions/runs/37120476610)：日志核验 173 项前端、45 项 Python、发布结构检查和 5 项负向检查；JAR 构建与测试通过。
- [Vue 3 37120476612](https://github.com/warpdotsys/reader-dev/actions/runs/37120476612)：已下载核心 JUnit 10 项，0 跳过/失败/错误，六本生成阅读完整旅程 18.490 秒。三张新增生成截图下载后再次目视核验。经理密钥文件与构建后的 `/reader/` 子目录通用旅程也实际执行并通过非跳过检查；后者不冒充子目录下的 EPUB Worker 阅读验收。
- [完整镜像 37120476657](https://github.com/warpdotsys/reader-dev/actions/runs/37120476657)：已下载实际 Camoufox JUnit 11 项、0 跳过/失败/错误，74.332 秒；合成脚本/POST、Cookie 序列和四账号请求通过。可选原件、旧引擎与公开页配对 jobs 没有启用，不记为通过。

源码 SHA 与制品使用的 PR 合并快照 `0ed5a430ceff37ceb8e586e045c00c08f2241374` 分别记录，见[制品摘要](evidence/epub-resource-budget-ci-c2892805-2026-10-03.json)。[原始资源 JSON](evidence/c2892805-browser-resource-budget-2026-10-03.json)与[合成结果](evidence/c2892805-browser-synthetic-2026-10-03.json)保留下载字节：2 CPU / 2 GiB / 256 PIDs，短时内存峰值 814,772,224 字节（约 777 MiB）、179 任务，OOM/限额事件为 0。采样属于镜像内 Camoufox 的顺序用例加四账号突发，不是前端 EPUB 解压的浏览器 RSS、长期负载保证或原 JAR 的资源峰值。

## 尚未验证与已知限制

- 此报告首先验证的是客户端原版排版预算；随后补充的[服务端 ZIP/EPUB 预算与失败导入保护](EPUB-SERVER-BUDGET-2026-10-03.md)另行记录。两侧字节限额都不是浏览器/JVM 总 RSS，仍需真实峰值与长期负载验收。
- ZIP 大小/路径检查不等于完整 CRC、所有 ZIP 方言或内容完整性校验。未证明所有 CSS 转义语法、字体整形、复杂媒体或任意大书兼容。
- 解压 Worker 有 15 秒上限；下载可以被用户取消，但尚无总下载时限，不能宣称网络挂起已全部覆盖。
- 本轮未重新运行原 JAR 的复杂样式界面，也未完成原 JAR/历史 renderer/内置引擎的真实登录书源同条件三方差分。
- 本轮三道托管门禁已实际通过，但没有发布 Release、推送候选镜像或切换生产；原生 ARM64 的历史候选记录也不冒充此次新增界面功能的 ARM64 验收。

## 重建与回滚

使用项目 Gradle Wrapper、JDK 11、Node 24 和锁文件，从仓库根目录构建：

```sh
npm ci --prefix web-vue3 && npm run build --prefix web-vue3 && ./gradlew -PreaderWebUi=vue3 test bootJar --no-daemon
```

本机 Node 运行时不附带 npm，已使用锁定的 `vue-tsc` 和 Vite 入口执行同等构建步骤；托管 runner 使用上面的 npm 命令。新代码不需要迁移数据。临时遇到原版模式限制可切换普通阅读；产品回滚保留前一完整镜像、配置与 `storage/data`，沿用部署文档流程，不覆盖原始 JAR、现有三个工程或用户未提交文件。
