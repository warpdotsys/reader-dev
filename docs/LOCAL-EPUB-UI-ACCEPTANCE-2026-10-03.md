# 本地 EPUB 阅读增量与隐私边界

日期：2026-10-03。候选未合并、未正式发布、未切换生产。

## 当前授权

网络书籍正文可用于测试。真实本地正文仅限用户指定的《黎明之剑》；用户随后明确批准将这一个 EPUB 复制到本机隔离 `build/allowed-local-book-20261003-a`。其他本地正文不读取、不复制；批量复制 59 本的待授权计划取消。可自行生成 TXT/EPUB 样本导入隔离服务。

授权 EPUB 为 37,809,796 字节，SHA-256 `bd8c68194bb26a3c3bc5edc7b6baa7023c584441f1c2835f71fab3a3bf3ced31`。复制后再次从服务器只读核查，文件摘要与该本书原有进度记录一致。文件、原始书架 JSON、正文与真实阅读截图不上传 GitHub。公开证据仅保留状态、数量、摘要及安全断言；托管浏览器回归使用完全生成的书籍。

## 已从原 JAR 实际验证

新脚本 `scripts/compare-authorized-local-reading-in-netns.py` 在真正的 Linux 私有网络命名空间内运行：验证内核网络 namespace 不同于 PID 1、仅有 loopback 接口，再降权为 nobody；在读取私有输入或启动 JAR 前拒绝普通宿主调用。校验原 JAR 固定 SHA-256，并将本地文件锁定为上面的唯一授权摘要。

两侧使用随机测试账号、独立临时目录与同一份原 EPUB，仅将该本书路径映射到各自的测试命名空间。原版 SHA-256 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`；本轮对照的恢复 JAR SHA-256 `ba1e3acb91f04bd6a3a10c7406a76e11cb82c1bf3d41212d22d14bef6953085c`，它是 UI 修复前已运行的基线，不冒充修复后产物。

实际执行结果：

- `getChapterList` 两侧均 HTTP 200、`isSuccess=true`、空 `errorMsg`，1611 项 index/URL/title 投影一致，标题 U+FFFD 数量为 0。
- 章节索引 0、1、805、1610 的 `getBookContent?epubContent=1&cache=1` 状态、ReturnData 字段、原 XHTML 与正文文本摘要一致，取样正文无 U+FFFD。第 0 项是无文字页，不将它误判为正文丢失。
- XHTML 资源两侧均 HTTP 200、`application/xhtml+xml`；历史 asset 路由会向解压副本注入 Reader 脚本。资源字节与注入后的 HTML API 相同，正文文本摘要不变，原 EPUB 不变。
- 重新生成目录会按原逻辑更新**一次性测试书架**的目录元数据；因此首轮“整个测试书架必须逐字节不变”的断言失败，未将失败记为通过。第二轮明确分开目录更新与阅读进度：目录生成后的缓存读取不再改变书架，原有四个进度字段不变；匿名及第二用户目录请求被拒绝。
- 输入原件、业务输入和两个 JAR 均未改变。没有生产进度写入，没有读取其他本地正文。

该取样不是其他本地书、全部章节、生产用户权限或 EPUB 原版 iframe 模式的验收。

[修复前基线汇总](evidence/authorized-local-epub-netns-2026-10-03.json)不包含私有原路径、标题列表或正文。

## 可见界面发现的 bug 与源码修复

内部浏览器实际进入授权 EPUB 的第 805 项，候选“纯文本”显示 `/book-assets/...xhtml` URL，只有一个 75 字符段落，未显示正文。后端缺省 EPUB 响应是资源 URL；旧 Vue 3 加载器把它当作普通 TXT 字符串，并可能保存至本机章节缓存。这不是原 EPUB 编码损坏。

当前修复在 EPUB 的文本与净化排版模式均请求 `epubContent=1`，文本通过 detached template 提取并解码实体，删除 head/title/script/style/嵌入内容，保留段落。该惰性 DOM 不挂入页面，不执行脚本或加载嵌入资源；EPUB 绕过此前可能污染的文本缓存，普通 TXT/网络书缓存路径不变。API 注释已改为真实 Java/Kotlin 契约。

新增 `Vue3PreviewLocalReadingTest` 在 GitHub 托管 runner 用生成的中文 TXT 与双章节 EPUB，操作预览导入与确认入架，验证默认正文而非 URL、实体解码、脚本/标题不进入文本、前后章边界、刷新位置及净化排版刷新。workflow 强制检查其 JUnit 为 1 项、0 跳过/失败/错误，并仅上传合成截图。

提交前本机 Vue 类型检查、Vite 生产构建、136 项 Node 测试、39 项 Python 安全/协议测试、浏览器测试类编译、发布流程静态门禁与 5 项负向测试均通过。修复后真实 UI 与最新托管运行结果需在下方增量记录，不能由编译成功推定。

## 修复后实际重建与浏览器取样

本机独立输出 `build/epub-reading-candidate/reader-4.0.7.jar` 已成功重建，SHA-256 `191667301e16a6b9760ae9beccfd3965be72b9fac9f2534eef968b57a8bff883`，没有覆盖旧基线 JAR。仅重启已核验归属的本机回环候选，未触及生产。

内部浏览器第 805 项从资源 URL 恢复为 66 段、3823 个渲染字符，第 806 项为 59 段、3901 字符。前后翻章更新 URL，刷新仍保持对应章节，加载结束后正文可见。净化排版显示 64 个 `<p>`，脚本数为 0，取样无 U+FFFD，刷新仍为该章节的 HTML 模式。技能驱动的真实取样发现了旧合成网络书旅程未覆盖的 EPUB 契约错误。

新修复 JAR 再次在仅回环 Linux 命名空间与原 JAR 对照，1611 项目录及同样四个取样 XHTML/正文、HTTP/ReturnData 和资源注入断言继续一致；只读原件、缓存阅读进度、匿名拒绝及第二用户拒绝均通过。工具现强制任何目录/正文差分或资源断言失败返回非零，新增负向回归；Python 合计 40 项。

`348315ec` 的首次托管新增阅读旅程失败于合成 EPUB 路径：Java `URLEncoder` 将书名空格转为 `+`，不是 Vue Router 的 `%20` 路径编码；生成 TXT 已完成，生成 EPUB 未发送正文请求。修正夹具路径编码并增加失败诊断，没有去掉正文断言、缩减夹具或把失败宣称通过。后续 run 结果另记。

真实继续操作还定位模式回退问题：在 HTML 模式刷新加载后，切回文本会显示“本章暂无内容”，再次刷新才恢复。修复让一次 XHTML 加载同时生成 HTML 与惰性文本投影，不需要重新下载；新增回归要求经过模式循环返回文本后立即可读。它不证明原版 iframe 下载成功，原版模式的明确失败继续列在下方。

模式修复单独重建 JAR 的 SHA-256 为 `8ccaa3093a3a017aacd12674d7b8c5e5dcf4332309bfc246a269382fa0bcb4b0`，输出位于 `build/epub-mode-candidate`。内部浏览器在第 805 项实际完成文本 → 净化排版 → 刷新 → 原版 → 文本，切回后未刷新即恢复 66 段、3823 个渲染字符。候选只监听回环；此截图不公开。

## 已下载核验的 GitHub 托管制品

修复提交 `2e48cc337611ec5388a77c9920164b2e5b9769e9` 的 [Java/Kotlin CI](https://github.com/warpdotsys/reader-dev/actions/runs/37102095391)、[Vue 3 浏览器旅程](https://github.com/warpdotsys/reader-dev/actions/runs/37102095377)和[完整单容器镜像](https://github.com/warpdotsys/reader-dev/actions/runs/37102095380)全部成功，执行标签均为 `ubuntu-24.04` GitHub 托管 runner。PR 制品对应合并提交 `edf798933f042cb4295e4e053e32b69fe8792cd9`。

下载核心 JUnit 为 10 项，0 跳过/失败/错误，新增 LocalReading 耗时 4.848 秒；生成 TXT/EPUB 均实际完成导入、阅读、翻章、刷新，EPUB 净化排版与返回文本继续通过。Camoufox 11 项无跳过/失败/错误，85.456 秒；镜像内合成 GET/POST、测试头、Cookie 序列及四用户请求通过。内存峰值 779,640,832 字节（约 744 MiB），PIDs 峰值 177，2 GiB / 256 PIDs / 2 CPU 下所有限额事件为 0；不是长期负载保证。

已逐项核验的[汇总证据](evidence/vue3-local-reading-2e48cc33-2026-10-03.json)记录原始 run URL、源提交与 PR 合并提交、JUnit、截图摘要及容器资源。仅下载合成截图，中文正文和实体解码可见；截图顶部保留触发已知原版下载错误的淡出提示，未抹去错误或宣称 raw 模式通过。40 项 Python 安全/协议测试及 136 项前端测试也由该提交的普通 CI 实际执行。

复现生成数据的托管旅程使用 `.github/workflows/vue3-preview.yml`，无需私有 EPUB、原 JAR、用户密码或宿主机浏览器运行时。授权真实 EPUB 的修复后后端差分另见[取样汇总](evidence/authorized-local-epub-restored-2026-10-03.json)。

## 回滚与尚未验证

恢复旧候选镜像/JAR 或切回保留的 Vue 2 入口，不回写这次临时测试目录到生产。原 EPUB 和原 JAR 保持只读对照。新版阅读逻辑绕过旧 EPUB URL 缓存，无需删除其他用户本机缓存。

历史已知 bug：旧候选实际切到授权 EPUB 的“原版排版”，报 `invalid zip data` 且 iframe 未出现。下载器把完整 `storage/data/...` 路径交给用户 home，还未处理目录 `index.epub` 及业务失败 JSON。此失败保留为历史证据，不把此前文本/净化排版取样冒充原版模式通过；本次源码修复及后续状态见下。

## 原版排版增量（89177211 / d1042966）

已从源码核验并修复：下载按本书 `originName` 映射当前用户 `__HOME__`、启用后的 `__WEBDAV__` 或 `__LOCAL_STORE__`，不使用管理级 `__STORAGE__`，不跨用户回退。只查询选中书自己的目录；目录布局下载 `index.epub`，单文件布局直接下载；显式携带隔离会话 token，识别 HTTP 200 的失败 JSON。无法下载时保留可见错误与重试，不再误报 ZIP 解压失败。后端权限、路径穿越和软链接检查没有改动。

解析器补齐 container 单引号属性；原版按 legacy TOC href 定位 ZIP 资源，不以目录下标猜测 spine。书内点击由宿主监听 iframe 文档，修复原先监听 iframe 元素无效和内链未解析目录的问题。原 CSS/图片/字体继续走离线资源；沙箱没有 `allow-scripts`，清除脚本、事件属性、跳转及表单，CSP 阻止外连。

已成功重建：第一次原版修复 JAR SHA-256 `a395f78618c521f19e05c2f482c90ba1a3bbd88f668a3b40e51bfc0ed6939a14`，独立输出 `build/epub-raw-candidate`。实际界面又发现 iframe 默认高度仅 150 px，随后修正可用视口。视口候选 JAR SHA-256 `b0f6ddec4dd03ea019ca300c071ce968ae27ef81aace2f9f365ad2d9099f15ef`，独立输出 `build/epub-raw-layout-candidate`，没有覆盖旧 JAR。

已实际操作：旧书架副本的路径属于原用户，新隔离用户下载明确拒绝，未放宽权限。通过导入对话框仅导入已授权的 EPUB 到隔离用户；同名提示选择新增测试条目，不覆盖旧书架记录，更不回写生产。预览 1611 项。授权文件 SHA-256 与原 JAR SHA-256均保持不变。

原版第 805 项 64 段、3952 个可见字符，刷新相同；实际下一章第 806 项 57 段、4016 个字符，再返回第 805 项。两项均无 U+FFFD 或脚本节点。新视口实际 566 × 852 px，第 805 项取样仍相同，不刷新返回文本恢复 66 段。只记录计数，真实正文及截图不上传。该 EPUB 的 OPF 为 2.0，1670 manifest 项、1611 spine 项，`dc:creator` 为 0 项，故文件重新导入显示“佚名”；保留原书架作者元数据，不伪造文件内作者。

本机前端 142 项、Python 40 项通过，类型检查及 Vite 构建通过。增强生成样本故意将 NCX 与 spine 顺序设为不同，实际验收原 CSS、书内切章、刷新、前后章边界及模式回退；最终本机 JUnit 1 项、0 跳过/失败/错误，22.719 秒，并新增 iframe 高度 ≥320 px 断言。仅生成的文本及原版截图纳入托管制品。

托管首轮 `89177211`：Java/Kotlin CI 和完整镜像成功，Vue 3 核心 10 项有 1 项失败，实际超时为原版刷新后上一章的正文等待，失败 JUnit 已下载；不将本机通过当作托管通过。后续 `d1042966` 保留同一正文断言并观察翻章请求/URL，补充视口断言和失败截图。首轮超时的精确时序原因未确认，不以一次重跑抹去失败。

已下载核验后续托管制品：[Vue 3 workflow 37104916139](https://github.com/warpdotsys/reader-dev/actions/runs/37104916139) 成功，10 份核心 JUnit 各 1 项，均 0 跳过/失败/错误，本地阅读 5.393 秒。额外安全文件和 `/reader/` 子目录旅程步骤成功。生成原版截图已目视检查，无错误提示，中文、原 CSS 和内链可辨；文本截图保留摘要，不冒充已目视复核。测试合并 SHA `f04b6c1068fea38da4000e77d6465b0dde48dc67`。执行 runner 标签均 `ubuntu-24.04`，不是自托管。普通 [Java/Kotlin CI 37104916136](https://github.com/warpdotsys/reader-dev/actions/runs/37104916136) 的日志实际记录 142 项前端、40 项 Python 测试以及正式发布流程的静态/负向安全检查通过。

[完整镜像 workflow 37104916134](https://github.com/warpdotsys/reader-dev/actions/runs/37104916134) 也成功，下载核验 Camoufox 11 项，0 跳过/失败/错误，86.759 秒；内置浏览器合成脚本 GET、POST 请求体/测试头、Cookie 清除/回放和四账户请求通过。x86_64 资源峰值 764,502,016 字节（约 729 MiB）、177 PIDs，2 GiB / 256 PIDs / 2 CPU 下限额事件为 0。三个可选历史来源/公开配对作业跳过，不宣称执行了本次三方实站对照或 ARM64。原始 run URL、逐项 JUnit、截图摘要及隐私边界见[本轮公开汇总](evidence/vue3-raw-epub-d1042966-2026-10-03.json)。

读后复核生产原文件仍为 37,809,796 字节，SHA-256 与复制前相同；只读探针只选这一本，探针期间书架字节不变。原阅读进度 index/time/pos 与取样前一致，生产账户和服务没有执行写操作。

仍未完成：没有覆盖同一 XHTML 多个 TOC fragment 的精确定位、iframe 内滚动位置刷新恢复及嵌套 CSS `@import`；当前原版只做独立 XHTML 章节取样，不能宣称所有 EPUB 全功能通过。解压仍在主线程，超大 EPUB 的资源边界和长时交互尚未验收。其他未授权本地正文不在范围。双架构正式 manifest、生产出口隔离/长期负载和可回滚发布仍未完成；不创建正式发布标签、不更新生产。

## 原版内滚动位置增量（f5e4764f）

修复前实证：上述视口候选 `b0f6ddec...` 仍在本机隔离运行时，增强生成书籍回归执行到 iframe 内滚动后等待 `/reader3/saveBookProgress` 超时，JUnit 1 项失败、0 跳过，39.402 秒，原始报告另存于忽略目录。不是原始 JAR 的失败结果。源码确认旧组件只发送比例，不保存内滚动；宿主恢复逻辑也错误应用外层像素位置。

源码修复：原版进度另存本机，键使用 JSON 元组编码隔离 namespace 与 bookUrl，避免连字符拼接碰撞；恢复要求精确 chapterUrl（含 fragment）一致且位置为非负有限数。iframe 在图片加载和有界字体等待后恢复内层滚动，旧章/卸载等待不得回写新章，并提供 `aria-busy` 恢复状态。用户内滚动触发既有进度保存，生成测试断言后台仍只接收 `{url,index}`，不新增或伪造 Rust 像素字段。外层滚动不覆盖原版比例；明确切章从顶部开始。后端 API/存储格式没有改动。

本机验证：前端 145 项、Python 40 项、类型检查及 Vite 构建通过。增强生成 EPUB 的两个章节各增加 50 段，均实际内滚动 600 px，检查保存请求/本机记录、刷新恢复 580–620 px，宿主位置 <80 px，再切章与返回文本通过。JUnit 1 项、0 跳过/失败/错误，16.517 秒。真实书仍只在原授权隔离账户，内部浏览器以点击和 PageDown 实际滚动：稳定后内层 2344.6667 px，刷新后恢复相同，64 段、0 U+FFFD/脚本节点，外层仅 64.6667 px；交互控制只观察渲染 DOM，不直接读取隐式浏览器存储或输出真实正文。主动下一章 806 为 57 段、内层 0 px；主动返回 805 为 64 段、内层 0 px，明确切章从顶部而非误用旧位置。独立 `build/epub-progress-candidate` JAR SHA-256 `97cb130670cba0a35f31a8b2966f9eb61e489232e7bc8ef6866cbef861a6e065`。

提交 `f5e4764f7ceededf2602e3d2c457fbb6414407ca` 的 [Java/Kotlin CI](https://github.com/warpdotsys/reader-dev/actions/runs/37106811266)、[Vue 3 界面旅程](https://github.com/warpdotsys/reader-dev/actions/runs/37106811182)与[完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/37106811208)全部成功，执行 runner 标签均 `ubuntu-24.04`。已下载该提交自己的核心 JUnit：10 项、0 跳过/失败/错误，本地阅读 6.104 秒；合成 TXT/EPUB 实际完成导入、读取、内滚动保存、两章刷新恢复、内链/前后章及模式回退。额外安全文件与 `/reader/` 子目录旅程步骤成功。PR 测试合并提交为 `3a6543dd24ee6be9ce415d22101c957a599a97cc`。

两张本轮生成截图均已目视核验，中文、实体、原 CSS 和内链可见，无错误提示；仅合成截图纳入制品。普通 CI 日志明确记录前端 145 项、Python 40 项、发布流程静态检查及 5 项结构/负向测试通过。下载 Camoufox JUnit 11 项、0 跳过/失败/错误，87.026 秒；镜像合成 GET/POST、请求体/测试头、Cookie 删除/回放与四账户请求通过。内存峰值 816,467,968 字节（约 779 MiB），176 PIDs，2 GiB / 256 PIDs / 2 CPU 下限额事件为 0；不是长期并发或生产性能保证。三个历史来源/公开配对可选作业跳过，没有本次三方实站对照或 ARM64 证据。[逐项公开汇总](evidence/vue3-epub-progress-f5e4764f-2026-10-03.json)记录摘要与隐私边界，不借前一提交的绿灯代替。

范围限制：原版像素位置只在同一浏览器本机恢复，不宣称跨设备同步；同一 XHTML 多 TOC fragment 的精确初始落点与嵌套 CSS、超大 EPUB 资源仍未验收。旧本机进度不删除，生产和原件不回写。回滚旧镜像/JAR 后新位置键会被忽略，不能通过删除其他书籍缓存来回滚。
