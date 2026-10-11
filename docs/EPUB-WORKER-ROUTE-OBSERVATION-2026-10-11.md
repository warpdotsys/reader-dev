# EPUB 实际阅读观察：整轮未通过，生成 Worker 接点已观测

更新：2026-10-11。此前 H/I/J/K 的压力、身份与超时失败原样保留，见[前轮诊断](EPUB-WORKER-DIAGNOSTIC-2026-10-11.md)。本次不是压力修复或完整八场景验收。

## 本次实测与边界

**已从 JAR 验证**：使用自己的托管 Native 运行 38127563397 下载、自己的双 clean 重建独立核对的 be87 JAR，285,672,055B、SHA `be87f00490fc53a0462c48b06cc5500c6fea80ee07ee3f48b2fde035eb83eeab`；源码 `3e225f97efe1036e405490e0b4b253f1285a7ce4`。浏览器仍来自旧锁定 runtime `401d895c…bc44`，不是最新完整 Native 镜像本机验收。

**已实际运行**：只使用授权《黎明之剑》一份 EPUB；新生成账号经界面注册、导入预览和保存后，顺序采样 text805 / html805 / first raw805。不读取原书架进度、不运行原 JAR/API 双方对照、不复制其他本地正文、不导入真实凭据、不发布正文/截图。

| 采样 | 字符 / 段落 | 替换乱码字符 / script | 有限结论 |
| --- | --- | --- | --- |
| text805 | 3953 / 66 | 0 / 0 | 该次文本就绪 |
| html805 | 3952 / 64 | 0 / 0 | 该次 HTML 就绪 |
| raw805 | 3952 / 64 | 0 / 0 | 该次原排版就绪，iframe 872px |

HTML/raw 正文 SHA 同为 `47f23ed8aa5dd68549872f3f73e4c0e971811248bff0abdd21afe5d4f06225f6`。该次没有可见下载/Worker 错误；这不否定 H/I 的真实 15 秒超时，也不等于刷新、连续换章、后端对照或生产容量通过。

**整轮仍失败**：生成 module Worker 自检有四事件，但主文档 route hook 仅见 fixture/login 两次。Reader 导航计数为 0；Reader `goto` 后立即及 raw 就绪后的两个 marker 均 missing。最终校验正确拒绝，最初错误为有限 `validation`，没有再被诊断/收尾异常掩盖。

**同轮完整资源**：父级包含 Reader、浏览器与控制进程；2 CPU、2 GiB、memory.high 1.5 GiB、256 tasks/threads、swap 0。峰值 1,621,876,736B / 248 tasks；PID/hard-memory/OOM 事件 0，soft-high 11,603。探针耗时 45.544 秒、单元运行 55.063 秒；两者不能混作业务 Worker 耗时。服务停止/MainPID0、两个自有容器 stopped/PID0/delete、inventory0 均观察到。

[公开有限回执](evidence/authorized-epub-be87-l-finite-diagnostic-2026-10-11.json)没有正文、原始 URL、Cookie、凭据、私有日志或临时账号。原始有限 JSON SHA `214714d2…1cd4` 留在本机；公开回执不是整份原始 JSON 的复制。

## 可维护探针变更，不是业务修复

`scripts/epub_worker_lifecycle_probe.py` 新 SHA `e4d2573dbb51e2259ef6299b681ba8c27337a79e33441323d96a41c38df3e81f`：Worker 包装本身未改；新增 Python/JS 一致的有限路径分类、route/navigation/html/injection 数值、CSP 有无及主异常保护。候选 URL 即时分类后丢弃，不输出 query/fragment；计数饱和到 4096，实际 harness 拒绝 cap-hit。Worker 仍限定 64 事件，仅字节量/时间/布尔，不输出文件名、正文或构造 URL。

25 条纯生成探针测试通过；本机完整 Python 725 项 / 64.353 秒，724 执行通过、1 项 POSIX 符号链接测试因 Windows 跳过。L harness 另有 19 项 AST/合成检查；不是托管该新提交的验收，不取代实际 Reader Worker 接点。

异常路径将 marker、资源、context/browser/Java 收尾分别降为有限诊断结果并保留 primary；独立核验仍明确 `accepted=false`。页面 HTML 注入有真实时序开销，不能声称无扰动或原产物时序等价。

## 尚未验证与下一步

现在只能确认 Reader 顶层导航没有派发给此 hook，不能把原因直接认定为重定向、缓存或 Firefox 跨进程。[Playwright 官方路由约定](https://playwright.dev/python/docs/api/class-browsercontext#browser-context-route)说明路由会停用 HTTP cache，而当前 service workers 已 block；本次没有缓存命中或跨进程事件证据。[Camoufox 官方隔离评估说明](https://camoufox.com/python/main-world-eval/)亦不能替代实际页面注入证据。下一步用纯生成文档比较 regex/glob、request/frame/goto 与 hook 的有限计数，先补齐观察链路，不再为此重复读取真实书。

不提高产品 15 秒 Worker / 60 秒下载上限，不增加合并预算，不改 main-world-eval、个人浏览器、映射、代理或生产配置。fflate 扩容风险、完整八场景、OPDS、同章 anchor 等已知问题仍未闭合。

## 清理与回退

停止与身份核对后，只 unlink 两个确切临时输入副本，共 323,481,851B，按绝对目录、UID、nlink、只读权限、大小与 SHA 校验；没有递归删除目录。临时副本可由保留的托管 JAR/授权 EPUB 重建，来源仍保留且复哈希一致。58 份用户脏报告与原始 JAR/只读备份的保护检查通过。

回退仅撤回这个测试探针及对应回归，不改业务 JAR、前端、规则、安全或资源上限；保留 L 失败与 H/I/J/K 原证据。没有合并、tag、发版、registry 推送或部署，goal 保持 active。

## 新提交的托管复核补充

**已成功重建并测试**：测试探针维护提交 `324a7d5057cb4734a21dd09ef6316167d5c03f1b`，实际 PR 合并快照 `6780ec625e332e0e8385513b527a152d895ef2a2`，四项必需门禁全部终态成功，并由独立有限回执校验及根代理只读重算全部守卫确认。重算精确比较既有回执，不覆盖原回执。PR 56 仍为 draft，默认分支仍为 `legacy`。

| 门禁 | 实际运行与有限结论 |
| --- | --- |
| [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/38130748738) | Linux Python 725 项零跳过；JVM 总计 207 项，其中 35 项既有环境跳过；4 项编码测试零跳过；两次 clean JAR 字节一致 |
| [Vue 3](https://github.com/warpdotsys/reader-dev/actions/runs/38130748757) | 实际界面 25 项零跳过；前端 304 项；截图保护 20 项 |
| [完整单容器](https://github.com/warpdotsys/reader-dev/actions/runs/38130748748) | Camoufox 24 项零跳过，Chromium 23 项零跳过；完整镜像生成账号登录、书架、阅读、退出；HTTPS 正例与负证书对照 |
| [原生两架构](https://github.com/warpdotsys/reader-dev/actions/runs/38130748737) | amd64 / arm64 构建、转移、实际载入和新生成数据运行；32 项有限 publisher 文件核验；没有推送 registry |

Java / Vue / Full / Native 同一 JAR 均为上文 be87。Full 的完整服务预算峰值 858,562,560B / 193 tasks；Native 有限回执各阶段最大值 956,137,472B / 202 tasks。这些托管生成场景不同于本机大型 EPUB 压力场景；客户端界面驱动位于服务预算之外，不能把两者拼成大型书籍容量结论。原 JAR 三方、真实站点登录和压力完整八场景仍未由这些门禁证明。可选专项任务的跳过也不当作对应专项通过。

根代理另外查看了本轮托管生成的登录、阅读及 TTS 编辑截图，未见这些截图中的乱码；这不代表生产界面或全部页面已无编码问题。Native 仅下载并校验小型 JSON/XML 回执；没有下载大镜像，也没有在本机重新计算其归档 SHA。全目标、生产变更和发布状态均仍为 false。

## M 生成文档诊断的启动失败

**已实际运行，但未进入浏览器**：仅生成页面诊断 M 于启动自有容器时失败，有限 primary 为 `validation`；没有挂载 JAR、EPUB 或真实输入。原 M 脚本及失败回执保留，不在原目录改写重跑。

只读检查确切自有目录显示 fresh / client-bundle / rootfs / verification 均为 `0700`、UID/GID 1000。结合该次生成专用启动错误，可确认 `umask(077)` 将请求的 `mkdir(mode=0755)` 收紧成 0700，导致容器内 UID 10001 无法遍历根目录并启动 Python。这是生成诊断 harness 的权限错误，不是实际 Reader 的业务错误，也不是 uidmap 映射错误。

M 的父级预算核验通过：2 CPU、2 GiB max、1.5 GiB high、256 tasks、swap 0；峰值 39,854,080B / 15 tasks，hard-memory/PID/OOM 事件均 0。探针 0.883 秒、单元 1.459 秒。自有容器 inventory 0、单元 inactive/MainPID 0；cleanup 的 `alreadyAbsent` 只说明未留下容器，不能冒充实际观察到容器 stopped/PID0/delete，更不能冒充 Worker 运行通过。

Ubuntu 官方 `uidmap 1:4.17.4-2ubuntu3` 已安装；newuidmap/newgidmap 可用，subuid/subgid SHA 仍与安装前一致。后续在独立 N 目录修正明确容器内目录的遍历权限，宿主 fresh 保持 0700，不改系统映射、个人浏览器、代理、生产、产品超时或资源预算。M 尚无 regex/glob 的实际路由结果，不能用此次启动失败推断 Reader 的路由缺口原因。

## N 保留失败；O 仅生成场景接受

N 修正容器可见目录为 0755 后，真实取得 host 与 inner 的 UID10001 / host UID110000、零 capability、no-new-privileges、仅 lo、六种不同于宿主的 namespace 及只读根目录。然而读取临时浏览器配置前失败，inner 有限 primary 为 `closed`，没有进入 generated probe；外层 config 字典断言又把顶层错误改成 `validation`。独立校验从保留的内层结果提取原始失败，明确拒绝 N，不能把 N 当作浏览器运行成功。N 峰值 48,803,840B / 14 tasks，无 hard-memory/PID/OOM/swap 事件；自有容器 stopped/PID0/delete、inventory0、单元 inactive/MainPID0 均观察到。N 原源文件与失败不改写。

O 在第三个独立目录继续修正：只对 runc 启动子进程使用 `umask(022)`，宿主 wrapper/fresh 仍为 077/0700；按成功 L 的结构设置三层 UID10001 私有 HOME/cache/camoufox tmpfs，配置/flag/浏览器仍只读挂载。原始 inner failure 先保存，后续 outer assert 另记 `outerDiagnosticCode`，不再覆盖 primary；配置错误仅记 permission/missing/io 的有限分类。没有调整系统映射、代理、个人浏览器、产品超时或总资源预算。

**已实际运行并独立核验 O 的生成子集**：regex 和 glob 两个新 context 各 3 次、总共 6 次 loopback 导航（login / 普通 Reader 路径 / 编码 Reader 路径），各次 HTTP 200、nonce marker、request/frame/navigation route 计数都实际收到。4 个生成 Blob module Worker 均有 created → posted → message → terminated 四事件，输入 123B、输出 77B。两份编码路径文档保留测试 CSP，未使用 main-world-eval。这里不是 Vue/Vite 的实际 EPUB Worker，也没有挂载/读取 JAR、EPUB、书架或凭据。

O 父级同预算：2 CPU、2 GiB max、1.5 GiB high、256 tasks、swap 0；峰值 1,611,108,352B / 165 tasks，soft-high 492，hard-memory/PID/OOM 事件 0。探针 12.927 秒、单元 13.604 秒；浏览器与生成 HTTP server 关闭、自有容器 stopped/PID0/delete、inventory0、单元 inactive/MainPID0 均观察到。soft-high 存在，不能把 O 当成内存无压力或大型 EPUB 时序等价证明。

[M/N/O 公开有限回执](evidence/camoufox-generated-route-m-n-o-finite-diagnostic-2026-10-11.json)保留三轮结果和 SHA。O 只排除了“这份生成文档下，regex 普遍失效、编码路径/CSP必然不工作”的说法；没有证明 L 的实际 Reader 页面为何未到达 hook。源码只读调查确定 L 最后使用同一 page 的显式 `/reader/...` `goto`，前端采用 `createWebHistory`，hook 提前绑定且 service workers block；不能仅用 SPA/hash 推断结案。下一步应先用自建 EPUB 复现真实 Reader 文档与 Vite Worker 接点，继续保留 L 与压力失败，不为此再重复读取真实书。

本节补充仍为本机文档工作区内容，未宣称已出现在 GitHub 的 `324a7d50` 源提交中。该提交四门禁的结果与本机 O 生成诊断分别绑定，不合并成生产验收；没有发版、部署或读取其他本地正文，goal 仍 active。

## P 实际 Reader 与 Vite Worker：接点已观测，整轮仍拒绝

**已实际运行**：同一 be87 托管 JAR、新账号和全生成的 806 章 EPUB，压缩 501,316B、810 条目、展开 6,718,908B。没有挂载原 JAR、真实 EPUB、原书架、远程 WebView 或真实凭据。隔离 UID10001、只读根/JAR/浏览器、仅回环、双容器只共享 user/net、原产品 15 秒 Worker / 60 秒下载和父级资源上限均保持；浏览器仍是旧锁定 runtime，不是最新完整镜像本机验收。

**测试方法不同且有扰动**：仅在自有隔离 namespace 内增加固定回环后端的 HTML 观测代理。禁止跟随重定向、核对最终 origin、CSP 原样转发；只注入公开页面的两个数值观察脚本。产品 Worker JavaScript 不改写，实际请求体 SHA 与所运行 JAR 内唯一 Worker 资源 SHA `46bb4b6d…2332` 精确一致。浏览器 route handler 全程 `continue_`，不使用 `fetch/fulfill` 或 main-world-eval。代理会读取生成数据响应并改变测试时序，不能声称原产物时序等价。

**已从实际页面观测**：注册、导入预览和保存均实际经界面完成；显式 Reader `goto` 返回 200，Reader 的 Worker/文档两个 meta 均 valid，当前文档 bootstrap 的 Reader 导航计数为 1。text805 / html805 / raw805 均 2891 字符，段落 46 / 45 / 45；script 和 U+FFFD 均 0，均包含生成 chapter0805 标记，raw iframe 为 641px，sandbox 为 `allow-same-origin`。该单次生成章不代表所有页面或生产编码已修复。

实际 Vite module Worker 收到该生成 EPUB 的 501,316B ArrayBuffer，严格四事件为 created → posted → message → terminated；原生消息收到 810 文件、6,718,908B、无 error reply。posted → message 为 275ms、created → terminated 为 276ms。这是实际产品解压接点，不是 O 的 Blob 自检；只说明该生成小输入，不能替代 H/I 大型 EPUB 的真实 15 秒超时或完整八场景。

**整轮仍失败**：P 后置守卫正确拒绝，保留 `passed=false`、原 `actualReaderWorkerLifecycleAccepted=false`、primary `validation`。浏览器 host route 的 Reader 导航计数仍为 0，尽管代理已送达真实 Reader 文档且页面 bootstrap 及 Worker 四事件齐全；代理还记录 `closed` / `validation` 两个有限错误，但没有采到其发生阶段，不能直接认定为正常取消、304 或业务故障。不能反写 P 成成功，也不能把观测接点的子集扩大为压力验收。

这次全程 `continue_` 的实际 Reader 仍有浏览器 route 缺口，因此“前序 fulfill 是唯一原因”已没有证据支持；L/O 相关性不能定因。下一步补固定有限 proxy 阶段/响应类别和独立 request/frame/server 计数，保留失败源文件与回执，不为这个观测问题重复读取真实书。

**资源与清理已观察**：父级峰值 1,611,587,584B / 255 tasks，soft-high 2401；hard-memory/PID-max/OOM-kill 0、swap 0。该峰值已逼近 256 tasks，不能称无压力、充足并发容量或无泄漏。探针 21.820 秒、单元 23.955 秒都不是 Worker 业务时间。代理、Java、sampler 已停止；两个自有容器实际 stopped/PID0/delete、inventory0，单元 inactive/MainPID0。58 份用户脏报告、原 JAR/只读备份及授权 EPUB仅复哈希，均未改动。

[P 公开有限回执](evidence/generated-reader-vite-worker-p-finite-diagnostic-2026-10-11.json)保留原失败和观测子集，不含正文、URL、Cookie、凭据、日志或截图。原始有限 JSON SHA `d787ce6f…e488`，只读保留。Windows 预检两次生成的 SHA 相同；本轮 Linux 只生成一次，实际输入 SHA 不同，大小/条目/展开量一致。两平台差异尚未逐字节核查，不能宣称跨平台字节一致，也不能把 Windows 输入 SHA 代入实际 Linux Worker 绑定。

独立只读验证器 SHA `50032e83…366c` 锁定五份源码、JAR/Worker/实际生成输入 SHA，严格解析 e4 的两个页面快照、四事件/单调时间、唯一容器清理及精确 cgroup/namespace。根代理重跑实际 P 回执和 9 项纯生成守卫（1 正例、8 拒绝）均得到预期结果：身份/资源/清理/三种 UI/实际 Worker 子集可确认，但 `wholeRunPassed=false` / `accepted=false`。公开回执又逐字段与原始有限 JSON 比较，不覆盖原件，也不把观察子集扩大为发布验收。

本节是后续诊断文档，不在 `324a7d50` 被测试快照内；四项托管门禁只对应上表既有快照。没有改产品、合并、tag、发版、registry 推送或部署，完整目标继续 active。
