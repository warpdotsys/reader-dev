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

## a4df 文档提交的四项托管复核

**已成功重建并测试**：文档提交 `a4dfb1a6de772da2248832741a123ced596212cd`，实际 PR 合并快照 `c2e271ff1f86b04d26dcc3f8341abff11099398b`。四项门禁均终态成功，独立回执及根代理只读重算一致；JAR 仍是同一 be87，不能把后续 Q/S/T 文档或 GET 修复算进这个被测试快照。

| 门禁 | 实际运行与有限结论 |
| --- | --- |
| [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/38144149410) | Python 725 项零跳过；JVM 报告总计 207 项，其中 35 项既有环境跳过，实际执行 172 项；编码 4 项零跳过；双 clean JAR 字节一致 |
| [Vue 3](https://github.com/warpdotsys/reader-dev/actions/runs/38144149408) | UI 25 项零跳过；前端 304 项；截图保护 20 项 |
| [完整单容器](https://github.com/warpdotsys/reader-dev/actions/runs/38144149492) | 实际 Camoufox 24 项零跳过；完整服务峰值 882,020,352B / 196 tasks |
| [原生两架构](https://github.com/warpdotsys/reader-dev/actions/runs/38144149453) | amd64 / arm64 构建、转移有限回执通过；各阶段最大 974,974,976B / 203 tasks；未推送 registry |

四份独立回执 SHA 分别为 `0d83adb2…831b` / `bdc313fb…15bb` / `02ddd4c8…95b1` / `f822535b…e315`。Native 未下载大镜像，保留 `localArchiveHashesRecomputed=false`。根代理查看了这次托管生成的登录/阅读截图，截图中的中文正常；这不是生产站全页面检查，也不是授权大型 EPUB、三方对照或完整目标通过。

## Q 定位阶段，S 改进测试工具：两轮均保留失败

**已从 JAR 和实际页面验证**：仍用同一 be87 JAR / 原样 Worker 资源 `46bb4b6d…2332` / 旧锁定浏览器 runtime；仅使用全生成 806 章 EPUB。固定 ZIP `create_system=3` 后，本机两次生成预检及两轮 Linux 实际输入 SHA 都是 `844e0c31…cb63`，压缩 501,316B、810 条目、展开 6,718,908B。这只证明这些具体样本，不反写 P 为跨平台一致。

Q 增加有限代理阶段与响应分类，并分别观察 server、浏览器 request、顶层 frame。实际三路 Reader 导航均收到：server=1、request=1、frame=3；原 host route 回调仍是 0。Reader 文档两个 marker 和 bootstrap 有效，实际 Worker 四事件输入/输出及资源 SHA 都绑定正确。Q 的有限失败明确是两次 `clientWrite/closed/200` 和一次 `backendStatus/validation/304`；不是未知阶段错误，也不能仅凭写入断开认定为正常取消或 Reader 业务错误。

S 在新目录仅修正测试方法：不再安装请求路由拦截；测试代理剥除 `If-None-Match` / `If-Modified-Since`、响应缓存校验/缓存头，添加 `no-store`。所有 3xx 仍拒绝，最终 origin、身份、原产品超时、预算、失败保护和实际 Worker 资源均不变；没有修改生产 HTTP 缓存或产品前端。五项纯回环 mock 核验缓存剥离、302/304、非法状态和错误 origin，不能代替真实运行。

S 实际 60 次 200、304/3xx/4xx/5xx 均为 0，三路 Reader 导航仍为 1/1/3；实际 Worker 四事件及三个 UI 模式均就绪。但仍有一次 `clientWrite/closed/200`，后置守卫正确拒绝。Q/S 的 `passed=false`、`actualReaderWorkerLifecycleAccepted=false`、primary `validation` 均保留；独立验证器只确认观察子集，不将新的三路导航守卫用于改写 Q，也不把 S 观察子集当作整体压力通过。

| 项目 | Q | S |
| --- | --- | --- |
| text / HTML / raw | 2891 字符，46 / 45 / 45 段 | 同左 |
| U+FFFD / script | 0 / 0 | 0 / 0 |
| raw iframe | 688px，`allow-same-origin` | 807px，`allow-same-origin` |
| Worker posted → message / created → terminated | 217ms / 217ms | 364ms / 364ms |
| 父级峰值 / tasks | 1,611,603,968B / 253 | 1,611,489,280B / 247 |
| soft-high 事件 | 2403 | 2339 |
| hard-memory / PID-max / OOM / swap | 0 / 0 / 0 / 0 | 同左 |
| 探针 / 单元耗时 | 23.849s / 28.077s | 24.601s / 26.772s |

探针/单元耗时不能混作 Worker 时间；soft-high 与接近 task 上限仍存在。双容器 UID10001、六 namespace 独立交叉核对只有 user/net 相同、只读根/JAR/浏览器及仅 lo 均观察到；没有真实输入挂载。两轮各两个自有容器 stopped/PID0/delete、inventory0，代理/Java/sampler 停止、单元 inactive/MainPID0 均确认。未读取真实书或凭据，也没有改动用户 Reader。

R 是两轮之间的启动失败，须单独记录：根代理未等辅助任务冻结便提前执行，Python `IndentationError` 阻止进入 Reader/浏览器；辅助任务同时修正了源文件。因此只有启动失败和单元 inactive/MainPID0 可报告，没有不可变完整回执，**执行源码绑定未验收**。不能拿后改的 R 源码冒充失败时冻结源码，更不能把它当作浏览器或预算测试通过。后续 S 在独立目录先完成全脚本 AST、`sh -n`、mock 和不再修改声明，再实际运行。

[Q/S 公开有限回执](evidence/generated-reader-worker-q-s-finite-diagnostic-2026-10-11.json)由冻结原始有限 JSON 逐字段生成并只读精确比较，不包含正文、URL、Cookie、凭据、日志或截图。Q 原始 SHA `211f74fe…13ba` / S `d89e425f…aaa9`；独立验证器最终 SHA `87d26169…703c` / `5e45843a…921d`，两轮都严格拒绝整体验收。58 份脏报告、原 JAR/只读备份和授权 EPUB 的哈希保护复核未变。

下一步只在生成环境加入“同一 request 对象 + 测试响应头”的短期内存关联，区分实际对应的浏览器请求失败与未关联的写入断开；不导出请求标识、URL、header 或错误原文。该诊断仍保留原 proxy failure，不通过放宽守卫来取得绿色结果。大型 EPUB 压力、完整八场景、fflate 临时扩容、原 JAR/远程 WebView 三方、真实站点登录、OPDS 与同章 anchor 等仍未闭合。没有合并、tag、发版、registry 推送或生产部署，完整目标仍 active。

## T 有限请求关联：没有证实取消，整轮仍失败

**已实际运行且保留失败**：T 使用同一 be87 JAR、原样 Worker 和 SHA `844e0c31…cb63` 的全生成 EPUB；先冻结源码、通过 AST / shell / 7 项纯回环 mock，再执行。仅测试代理增加短期内存 trace 与同一浏览器 request 对象关联，公开回执不包含这些标识、header、URL 或正文。无路由拦截、仅回环、缓存卫生、3xx 拒绝和原产品超时/资源上限均保留，没有改产品来迁就测试。

实际 59 次 200，304/3xx/4xx/5xx 均 0，仍有一次 `clientWrite/closed/200`。清理前后两个关联快照完全相同：API 类 server 写入失败 1、浏览器请求失败 2，**精确关联为 0**，全部未关联。不能把同为 API 类当作同一请求，也不能把未关联推定为正常导航取消。原始事件失败记录字段缺失，公开回执保留 `eventFailureRecordsFieldPresent=false` / `null`，不把缺失伪造成空列表；该字段只记录 request/frame 观察回调自身的异常，不是浏览器 `requestfailed` 列表，不能称浏览器零失败。冻结验收器对该字段使用空列表默认值，因此明确拒绝原因是非空 proxy failure，而非“缺字段也被严格拒绝”。后续新探针须显式初始化该字段，并单独记录有上限的浏览器失败类别/有限码；不能反写 T。

独立验收器 SHA `7e3f87a7…fb63` 的 15 项纯结构守卫通过，根代理也实际重跑。源码、身份/namespace、父级资源采样、三个 UI 模式、原版 Worker 接点/资源、三路导航及清理守卫均通过；`finiteProxyAndEvents=false`，`passed=false` / `accepted=false` / 原 `actualReaderWorkerLifecycleAccepted=false` 和 primary `validation` 均保留。生成文字 2891 字符、46/45/45 段、U+FFFD/script 0，raw iframe 582px；实际 Worker posted→message 157ms、created→terminated 158ms。它们仍只是该次生成样本的观察子集，不是大型 EPUB 或全项目验收。

父级峰值 1,611,603,968B / 249 tasks，soft-high 2369，hard-memory/PID-max/OOM/swap 0；探针 22.037 秒，不能混作 Worker 时间。两个自有容器 stopped/PID0/delete、inventory0，代理/Java/sampler 停止；根代理另外确认单元 inactive/dead/MainPID0。没有读取真实正文/账号或更改用户 Reader。

**从当前源码核查，尚未定因**：登录成功后才导航，导入/入架调用及书架卸载均没有 API 主动取消信号；Reader 的 AbortController 仅覆盖 EPUB 加载，不能据此解释上述 API 写断。另发现 GET 包装器声明支持 `signal` 却未透传，须单独修复和测试，不能称它已解释 T 的故障。缓存 runtime 的 `release` 元数据是 Eclipse Adoptium Temurin `11.0.32.1`；本机编译 JDK `11.0.8` 不是该 runtime。两 CPU 配额不等于 JVM 仅感知两个处理器，尚未做实际 JVM 线程/处理器归因，不据此擅改线程池参数。

[T 公开有限回执](evidence/generated-reader-worker-t-finite-diagnostic-2026-10-11.json)逐字段绑定原始只读 JSON SHA `c68d6948…229a`，并由根代理只读精确比较。Q/S/T 文档属于后续快照，不属于 a4df/c2e 四绿快照；未合并、发版、registry 推送或部署。完整目标及上述主要风险均未闭合。

## GET 取消信号窄修复：源码与本机回归，自己的托管构建待验

**已从当前源码确认并修复**：`web-vue3/src/api/request.ts` 的 GET 包装器没有透传已声明的 `RequestOptions.signal`；本次只向原 Axios config 增加 `signal: opts?.signal`。GET 的 params、headers、silent、timeout 和返回值、POST 既有判别/透传、登录/命名空间与后端存储逻辑不改。这不是 T 写断的已证实根因，也没有提高产品下载或 Worker 超时。

**本机回归通过，不等于锁定重建**：新增 `requestSignal.test.ts` 四项：GET signal 及原参数/header/timeout、无 signal、POST 既有 signal、真实 Node HTTP adapter 取消。以 Vite SSR 载入实际 `request.ts` 和实际 Axios；测试插件只替换 router/message/store/requestContext/namespace 等边界依赖，不用伪浏览器或假取消代替 HTTP 管线。最后一项显式选择真实 `http` adapter、禁用环境代理、只绑定 `127.0.0.1`，实际请求到达后 abort，验证 `ERR_CANCELED` 和客户端关闭；响应为自生成常量，未读正文/凭据。相关请求/session/error 共 16 项、完整前端 308 项及 TypeScript 检查通过。根代理实际重跑了 16 项、全前端 308 项和类型检查；本机使用已有依赖目录，不宣称执行了 `npm ci`。新提交、实际 PR 合并快照及自己的 GitHub 托管构建须另行绑定，不能借 a4df 的旧四绿发版。

**另一个已知缺口，尚未修改**：POST 包装器把第三参数判为选项时只识别 silent/signal/headers，因此仅 `{timeout: ...}` 会被当成旧 query params。当前相关 wrapper 调用同时带 silent，纯 timeout 的文件/导入调用直接走 Axios instance；未确认这些现有业务受此缺口影响。暂不顺手改变旧 query 判别语义，后续须单独明确兼容和测试；不能称所有 RequestOptions 已正确支持。

回退该产品增量时，在独立干净工作树只撤回 GET 的 signal 透传并形成单独回退提交；保留诊断及已知问题记录，不 reset/checkout 用户工作区。回退会重新失去 GET 取消能力，不能称业务缺口消失。真实大型 EPUB、完整八场景、资源/写断归因、原件/远程 WebView 三方及生产切换仍未完成，当前没有合并或部署。

## 2026-10-12：bfd 自己的托管构建通过，V 实测仍保留失败

**已成功重建并独立复核**：GET 修复提交 `bfd30ebf5e5eda318962a4380a9032fdec470751`，实际测试 PR 合并快照 `06bf1117a4baffffaab23e047d542eaee47a2a89`；不是上述 a4df/c2e 证据。以下四项均为 GitHub 托管 runner 终态 success，根代理重新执行全部冻结守卫并只读精确比较既有回执，未覆盖回执：

| 检查 | 本次证据范围 |
| --- | --- |
| [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/38148738554) | Python 725 项；JVM 总计 207 / 35 项既有环境跳过 / 实际执行 172；双 clean 完整 JAR 字节一致 |
| [Vue](https://github.com/warpdotsys/reader-dev/actions/runs/38148738557) | 前端 308 项；生成浏览器旅程 25 项 |
| [完整单容器](https://github.com/warpdotsys/reader-dev/actions/runs/38148738575) | 实际 Camoufox 24 项；合成子集峰值 882,098,176B / 197 tasks；四张生成登录/书架/阅读/退出截图实际查看且重算哈希 |
| [原生两架构](https://github.com/warpdotsys/reader-dev/actions/runs/38148738570) | amd64 / arm64 构建、转移和重导入有限报告通过；publisher 子集最大 904,609,792B / 202 tasks；没有 registry 发布 |

实际下载并逐条哈希的 JAR 为 `0675ec6dd851f737804e0f6d306b265f0b9e9b5df45fb62f79c9ad018aa2febe`，285,672,106B、1570 条目。与旧**恢复版** be87 JAR 完整 entry 内容比较，69 项差异全部是 Vue：index 内容变化 1 项、哈希资源新增/删除各 34 项；Java/Kotlin class、依赖、非 Vue 资源、manifest 内容差异均 0。这不代表与原始 b26 JAR 已完全等价。未下载或本机重算大型镜像归档；四绿也不抵消下述生成诊断、大书压力或生产未验证。

**U 没有运行**：冻结代码审查发现仅拒绝 browser overflow、未拒绝非空失败记录/非零计数的假成功风险；另把关联器 `server_write_failed=false` 的“尚未关联”误写成 capture 错误。U 四源及独立工具全部保留，不能称 U 已实测。V 在新目录修正成功与清理后的失败守卫，并保留原 primary；无资源/超时放宽。V 的平铺值由严格 after 快照直接赋值，内部 cleanup helper 的相等比较并非任意输入类型验证；独立根验收器另对平铺类型、before/after、额外 helper SHA 和新 JAR 身份严格检查。原独立 V 工具仍残留旧 JAR 常量/字节数且新字段检查不足，原字节保留，不使用它给出正向验收；根验收器锁定原工具 SHA、只精确更新运行身份并收紧字段，49 项纯结构护栏实际通过。

**V 已实际运行、整轮拒绝**：使用 bfd 的 0675 JAR，但 browser/runtime 仍是旧缓存 `401d895c…bc44`，明确不是本次最新整镜像本机验收；只有同一全生成 806 spine EPUB，501,316B / 810 ZIP 条目 / 展开 6,718,908B，SHA `844e0c31…cb63`。text/HTML/raw 都读到 2891 字符、46/45/45 段、U+FFFD/script 0，raw iframe 653px。Worker 原资源 `46bb4b6d…2332`、四事件、posted→reply 143ms / created→terminated 144ms；server/request/frame Reader 导航 1/1/3，host 0。这是观察子集，不是完整八场景或原件三方。

实际两次 API 类 `abort` 和两次 `clientWrite/closed/200`；两个清理前后快照相同，但 **精确关联仍为 0**。观察回调自身异常记录显式存在且空，不能等同浏览器零失败。另有 5 个有限 correlation capture `validation`，不能静默丢弃；现有分类还不足以解释这些捕获拒绝或证明对应请求。相同 API 类、相同数量不能证明一一对应，也不能据此宣布正常取消或业务修复。当前 EPUB 下载走原生 fetch，章节/目录调用未传 signal；本次共享 GET 修复不能直接解释或修复该次写断。下一步须在生成环境记录有限操作名/资源类型及绑定拒绝原因，不输出 URL/query/header/标识或正文，不反写 V。

父级峰值 1,611,587,584B / 246 tasks，soft-high 2512，hard-memory/OOM/PID-max/swap 0；探针 21.768s / UI 探针 19.016s / systemd 单元 26.043s，不混作 Worker 时间。保持 2 CPU / 2 GiB / high 1.5 GiB / 256 PID / 零 swap，身份/namespace/只读挂载/仅 lo、实际 Worker、清理守卫独立通过，browser failures 与关联守卫拒绝。两个自有容器 stopped/PID0/delete、inventory0，代理/Java/sampler 停止，另外实查单元 inactive/dead/MainPID0；没有真实正文或凭据，也没有生产更改。

[V 公开有限回执](evidence/generated-reader-worker-v-finite-diagnostic-2026-10-12.json)绑定原始只读 JSON `ab5e7a51…20af` / 124,240B。原始 `passed=false`、独立 `accepted=false`、product/pressure/full-goal 全 false 保留。官方 Ubuntu uidmap `1:4.17.4-2ubuntu3` 已安装，`/etc/subuid` 与 `/etc/subgid` 哈希仍为原 `d796e52b…e9cbd`；58 份用户报告、原件/只读备份及获准 EPUB 的哈希复核未变。此后文档属于后续工作区快照，不冒称已包含在 bfd/06bf 的四绿源码里；未合并、发版、部署或扩大验收范围。

## W/X/Y 定位与 Z 有限生成旅程通过：测试修正，不抹掉旧失败

**已实际定位**：W 在新目录增加只有白名单的操作名、GET/POST 等方法、资源类型和绑定拒绝类别；无 URL、参数、header、标识或正文输出。它记录一次 `getBookshelf` GET/XHR 的 abort 和一次 proxy `clientWrite/closed/200`，但精确配对仍 0，不能宣称是同一个请求。代码核查发现探针在收到 `saveBook` 响应后直接 `goto`，而界面随后仍 `await load()` 刷新书架；仅等待 `.bookshelf-page` 已存在不能证明导入工作完成。

X 起必须等导入弹窗在刷新后自动关闭、生成书卡片实际出现才跳页；没有提高产品或探针默认超时。X 的 API abort 与 proxy 写断没有再次记录，但出现一次图片请求 `other`，未定因，不能称图片 bug 已修复。Y 没有浏览器 requestfailed；两轮 proxy failure 记录字段均缺失，摘要保留 `fieldPresent=false/null`，不伪造成原始空数组。旧 proxy 守卫允许该字段缺失时用空默认值，不能声称它是“缺字段严格拒绝”。

**实际关联拒绝，不是捕获异常**：Y 的五条拒绝均为测试标记已赋给另一 request 对象；四条脚本响应 `from_service_worker=true`，一条 logo.png 为 false。Y 原字段 `network` 只表达该 bool 为 false，不证明一次新网络传输；Z 改名为 `non-service-worker`。Service Worker 的 Cache API 会保留克隆响应的测试 header，代理的 HTTP no-store 不等于禁止 Cache API。logo.png 的原源码和 JAR 字节一致，SHA `ab35c6f6…5c3a`，PNG 各块 CRC、像素解码和 512×512 尺寸实际通过；未改图，也不把 X 的间歇失败归因于坏 PNG。

Z 只修正诊断把 `bind_response=false` 当作 capture exception 的错误：对有当前内存标记、`token-other-request`、HTTP GET、已知静态 script 或 logo/favicon image、明确 SW/非 SW 来源的拒绝，保留每条有限记录但不伪报捕获异常。未知标记、缺 header、API/XHR、未知来源、POST、畸形输入、任何 requestfailed/proxy 写错、真正 callback/capture 异常、溢出仍拒绝；没有改业务前端、后端、原产品超时、cache policy 或资源上限。15 项纯 helper 回归及 72 项独立结构守卫真实执行，包含上述负例；不能把这个规则用来给旧轮次重新验收。

| 轮次 | 浏览器失败 | proxy 写错 | 绑定拒绝 | 该轮独立结论 |
| --- | --- | --- | --- | --- |
| W | 1 个书架 GET/XHR abort | 1 个 closed/200 | 5 | 拒绝，原始 `d344826a…63fb` |
| X | 1 个 image/other | 未记录，字段缺失 | 5 | 拒绝，原始 `10bc4f61…076f` |
| Y | 0 | 未记录，字段缺失 | 5，已细分类 | 拒绝，原始 `a29fa74f…42ab` |
| Z | 0 | 未记录，字段缺失 | 5，全部已知静态标记复用且保留 | **仅有限生成旅程接受**，原始 `b29c028f…5a63` |

Z 确实完成 UI 生成账户/预览/保存、刷新后可见、text/HTML/raw 三模式；每模式 2891 字符、46/45/45 段、U+FFFD/script 0，raw iframe 631px。原 Worker 四事件及资源 `46bb4b6d…2332` 绑定，posted→reply 214ms / created→terminated 215ms；server/request/frame 1/1/3、host 0。原始和独立 `accepted=true`，但 `productAcceptance/largeEpubPressure/fullGoalComplete=false`。原验证模板在成功时也有 `failure=other` 缺省值，它只在拒绝时才有诊断意义；Z 原始 primary 是 null，不把该缺省值当作新故障。

Z 59 个 200、1 个 4xx、304/3xx/5xx 0；已有状态守卫允许有限 4xx 记录，该条尚未逐路定位，**不能称所有 HTTP 零错误**。峰值 1,611,620,352B / 249 tasks / soft-high 2509，hard-memory/OOM/PID-max/swap 0；2 CPU / 2 GiB / high 1.5 GiB / 256 PID 不变。soft-high 与很小的 task 余量仍是风险，不是容量修复。探针 23.787s / UI 21.728s / 单元 24.871s 不混作 Worker 时间。W/X/Y/Z 都 stopped/PID0/delete 两个自有容器、inventory0；Z 又实查 inactive/dead/MainPID0，无真实输入、凭据或生产更改。

[W/X/Y/Z 公开有限摘要](evidence/generated-reader-worker-wxyz-finite-diagnostic-2026-10-12.json)逐字段从四份冻结只读记录投影并重新执行各自独立守卫精确比较，公开文件 SHA `8ca465db…370e` / 27,144B，未覆盖任何旧源码或回执。仍是 0675 JAR + 401d 缓存 runtime + 全生成输入，不是最新整镜像本机、授权大书、完整八场景、实站认证或原件三方；源产品没有为此增补空实现。

**已维护进真实 CI 测试源码，真实执行待本提交自己的托管检查**：`NativeImageDefaultUiTest` 保留原登录、错密、记住/不记住、TXT 翻章刷新、退出和四截图旅程，新增 2 章生成 EPUB 的 UI 导入、文本/HTML/原版、原版翻章与刷新；TXT 导入同样等真实刷新和书卡片，不只看 save 响应。JUnit 方法数仍 1、180s 总上限和 30s 默认等待未提高，完整 Reader 仍由现有镜像 job 提供，Chromium 只是预算之外的验收客户端，不是部署依赖，不替代合计预算验证。

本机 JDK11 `compileTestJava` 已通过；实际调用维护的 fixture factory/validator，6 条目 CRC/UTF-8/manifest 校验和两次独立生成字节相等通过（2107B，SHA `5ff6acb9…13b9`），固定 DOS 本地时间而非依赖时区。此夹具检查没有启动 Reader/浏览器，不能称新增镜像 EPUB 旅程本机已执行或借 bfd 的四绿通过。后续自己的 GitHub 运行和准确产物必须另绑定；主要未闭合项与原失败保持。撤回这一测试增量仅会减少自动覆盖，不修复业务问题；在独立干净工作树做定向回退，不 reset 用户数据。
