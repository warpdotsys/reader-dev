# 无凭据固定公开书籍的内置浏览器验收

日期：2026-10-11。首轮托管实站已实际失败；作者选择规则的后续修正尚待新的实站执行。目标是在已独立验收的单 Reader 镜像内通过 Java/Kotlin API 实际读取固定公开书籍详情、目录和首章，不能只凭站点直连成功或测试夹具证明产品可用。

## 范围与依据

选择 Project Gutenberg 第84号《Frankenstein》，固定[书籍说明页](https://www.gutenberg.org/ebooks/84)及[官方在线正文](https://www.gutenberg.org/cache/epub/84/pg84-images.html)。[站点 robots](https://www.gutenberg.org/robots.txt)禁止 `/ebooks/search`，探针不请求搜索、批量遍历、账号、验证码、购买或其它私有资源。

本机直接 HTTPS 取样正文页面 HTTP200 / 468,370 B；两次 SHA 相同：23403a6a3b1cace42a06d6c378a6f1acf8ec97c940c652d12fe3c0d30275e58f。确认29个章节 div、固定目录起始锚点 letter1/letter2/letter3/letter4/chap01。这个站点直连/结构证据**不是 Reader 实际执行结果**，不把直连通过作为内置浏览器通过。

探针只用生成 Reader 账号、新隔离 storage 和保存后逐项读回的专用书源；从页面本身提取标题、作者、28项目录和第一个 Letter 正文，不注入伪造页面/元数据或空实现。三个业务 API 各尝试一次；失败立即停止后续业务，不重试或延长产品单请求超时。输出只有有限字段、状态、字符数、SHA、安全和资源断言，没有完整正文、匿名站点 Cookie 或原错误。

仍需原件/历史 WebView 同输入对照、真实站点认证和其它目标；本模式不声称补齐三方或真实登录。原件不上传 GitHub，不使用已清除的起点 Cookie，也不读取其它本地正文。

## 镜像身份与重复命令

使用 Native 成功任务38111391710、测试快照ccbdad907fb803ca4f75a38d182cb86009372dc9、JAR9b9d0c9808968c0390022156a5fcbfcc03f595ed228f124afbb06e830cd7dd84。维护脚本是后续独立 checkout；不改写输入镜像来源、拿输入镜像绿灯接受新探针，也不在本模式重建或上传 registry。

复用 `.github/workflows/browser-image.yml` 的显式 `gutenberg-fixed-book` 模式；普通 Full 和起点 metadata 模式不变。仅 GitHub 托管原生 runner、2 CPU / 2 GiB / 256 PID / 零swap、UID10001、cap-drop ALL、no-new-privileges、零宿主端口和私网拒绝。重新校验下载/载入/运行的准确归档、镜像与 JAR 身份，观察实际 worker/browser 进程；有成功外壳但缺书名/作者/目录/正文、资源失败、会话清理失败或未观察浏览器均判失败。

```powershell
gh workflow run browser-image.yml --repo warpdotsys/reader-dev --ref ci/full-reader-20260926 -f public_metadata_native_run=38111391710 -f public_metadata_revision=ccbdad907fb803ca4f75a38d182cb86009372dc9 -f public_metadata_capture=gutenberg-fixed-book -f native_arch=amd64
```

执行条件是指定原生 artifact 未过期。若原生版本已改变，脚本的准确 JAR 守卫必须先经审阅更新和另验，不能换入不明 `latest` 或修改期望以掩盖失败。

## 已知问题和回退

### 首轮真实失败与独立观察

[实站任务38113464255](https://github.com/warpdotsys/reader-dev/actions/runs/38113464255)，源d6b7d6a43a6719f6fd51ac85bfe6ab724fd9d989，只执行一个 `/getBookInfo`：HTTP200 / isSuccess=true / 空errorMsg / data对象 / nameMatches=true，但authorMatches=false，按 MissingRealMetadata 判红；目录和首章均未执行，不计作三项通过。生成会话已撤销、Cookie索引0→0、容器已移除。5.425秒，实际worker1 / driver1 / browser7，峰值851,341,312 B / PID180，2CPU/2GiB/256PID，配置和实际swap0，无hard/OOM/PID触限。报告预算scope是最多三项的预定范围，实际执行数以 businessApiCalls 为准。

下载/载入/运行身份均是ccbd快照、9b9d JAR、镜像aa3b64d75108514eb9d1a1ee49dc97f80ba42e447aa7de008b6c5f634f47f8fc，输入Native38111391710/8b25。七个其它条件作业跳过，不计本轮真实执行。九JSON的精确集合/散列、源任务和托管runner、身份、完整有限响应、预算和清理已独立核对。[失败回执](evidence/gutenberg-first-independent-failure-2026-10-11.json)，SHA6b634318a8034cd8356435ffa8dae8dc03371f4f95c3d86d81d3f1c517fadcb7；[原观察](evidence/gutenberg-first-raw-observation-2026-10-11.json)与[原预算](evidence/gutenberg-first-raw-budget-2026-10-11.json)不改写。

### 基于结构证据修正来源规则，不改产品

后续本机直接取样仍为相同468,370B/SHA页面：h3只有25字符且不含Shelley；唯一以 `by ` 起始的h2包含作者。原探针把副标题h3当作者，已改为明确CSS `@CSS:h2:matches(^by\s)@text`，不注入作者名或删除作者校验。原CI没有保存网页字节，不能追补它与本机页面的完整相等证据或认定这是原轮唯一引擎根因。

新增两项真实JVM解析回归，生成标题/h3副标题/h2作者和缺作者页面：正确选择作者，缺失仍空；所在测试类18项全部执行、零失败/错误/跳过。[本机原XML](evidence/gutenberg-author-rule-local-2026-10-11.xml)，SHA01362dfec4e0300b60494cbe63c32beace9c0b0277ebc8976c663ba51b603a7e。Python专用守卫10项、完整637项 / 29.156秒 / 一个既有Windows跳过通过。仅维护来源规则及测试，未改变产品；旧失败保留，新托管实际结果另验，不借d6普通绿灯接受该修正。

初始d6探针本机新增九项离线守卫通过；完整 Python636项 / 29.242秒 / 零失败 / 一个既有Windows POSIX跳过，Node119项全部通过，YAML解析和脚本语法检查通过。最初全量回归发现旧固定模式枚举断言未同步；更新为五个明确模式，继续检查原默认值、原起点分支及无插值传参，不移除安全断言。本机离线通过不作为实站业务通过。

### d6自己的普通托管结果，不抵消实站失败

源d6b7d6a43a6719f6fd51ac85bfe6ab724fd9d989 / 实际合并快照fd515aff059a589a904448e77414ed5dca15f907：

| GitHub托管任务 | 实际执行及独立回执 |
| --- | --- |
| [Java38113466275](https://github.com/warpdotsys/reader-dev/actions/runs/38113466275) | Linux Python636 / 零跳过，JVM201 / 35既有环境跳过，双clean同9b9d；[Java回执](evidence/gutenberg-normal-hosted-d6b7d6a4-independent-java-acceptance.json) |
| [Vue38113466300](https://github.com/warpdotsys/reader-dev/actions/runs/38113466300) | 前端301，生成UI25 / 零跳过，11合成图身份；不声称本轮逐图目视；[UI回执](evidence/gutenberg-normal-hosted-d6b7d6a4-independent-ui-java-acceptance-v2.json) |
| [Full38113466335](https://github.com/warpdotsys/reader-dev/actions/runs/38113466335) | Cam24、实际打包Reader HTTPS和完整默认UI；峰829,956,096B/PID195；[Full回执](evidence/gutenberg-normal-hosted-d6b7d6a4-independent-full-acceptance.json) |
| [Native38113466292](https://github.com/warpdotsys/reader-dev/actions/runs/38113466292) | 六作业终态、双原生和重导入实际运行、publisher32有限报告逐字段接受；最大915,451,904B/PID202；[Native回执](evidence/gutenberg-normal-hosted-d6b7d6a4-independent-native-acceptance.json) |

自己的两个原生镜像是AMD64 464d88d8ecf96b3e25da4923365f51f9193f277eff447102ab05382a00d3c4fc、ARM64 41564046a1d1a79793eb60025f8cd2b730a92fc0f929a88c34c1618e0813c813；不是前述Gutenberg输入8b镜像。四回执SHA依次35ee77a3d569a602f25cbb514bc0749624d98a7fe2e6ee55485a198001dd1282、fc7fc61ad2f7e564580787c43659a1328ea41665553f747da6223786dd68b368、c3a939de3a75eb2773a45c97e90405d2a0d7e368a0146b052ec7140b2f4eb542、04152d322fcce10398947e4be7dfce69666bd3dca372c3292c1e56f4ae1134f0。

普通Full允许1GiB swap而实际0，不改称配置禁止swap；外部UI客户端不属于服务端预算。仅下载有限报告，原生归档摘要保留托管观测值，不冒称本机重新散列大型镜像。58受保护用户报告与准确原件未变。普通CI不运行此显式真实网络模式，不能把普通绿灯当本模式通过，也不能用d6结果接受后续作者/前端候选。

修正后的源规则尚待本镜像新的实站执行，包括目录带 WebView 选项的 URL、原始页面片段章节以及 Rhino `chapter.url` 的真实解析。不能因十项离线守卫通过就称业务已通。站点页面和上游网络也是变量；单次失败不自动归因于 Reader，也不通过关 TLS、放开私网或处理验证码取绿灯。

此前起点、认证头、合并资源预算超时等失败仍保留；本次不发版、不合并或部署。回退只撤回新增显式模式和自有探针，旧起点模式及其失败记录保留；仅清理本轮自有容器和一次性 storage，不动生产或用户工作树。
