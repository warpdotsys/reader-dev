# 无凭据固定公开书籍的内置浏览器验收

日期：2026-10-11。第三轮准确新f5c9镜像已实际通过详情、完整28目录和首章6822字符；前两轮真实失败保持不变。不等于原件/历史WebView三方或真实认证通过，也不提前接受后续真实UI候选。cc17自己的普通CI另有独立回执。目标是在已独立验收的单Reader镜像内通过Java/Kotlin API实际读取固定公开书籍，不能只凭站点直连或夹具证明产品可用。

## 范围与依据

选择 Project Gutenberg 第84号《Frankenstein》，固定[书籍说明页](https://www.gutenberg.org/ebooks/84)及[官方在线正文](https://www.gutenberg.org/cache/epub/84/pg84-images.html)。[站点 robots](https://www.gutenberg.org/robots.txt)禁止 `/ebooks/search`，探针不请求搜索、批量遍历、账号、验证码、购买或其它私有资源。

本机直接 HTTPS 取样正文页面 HTTP200 / 468,370 B；两次 SHA 相同：23403a6a3b1cace42a06d6c378a6f1acf8ec97c940c652d12fe3c0d30275e58f。确认29个章节 div、固定目录起始锚点 letter1/letter2/letter3/letter4/chap01。这个站点直连/结构证据**不是 Reader 实际执行结果**，不把直连通过作为内置浏览器通过。

探针只用生成 Reader 账号、新隔离 storage 和保存后逐项读回的专用书源；从页面本身提取标题、作者、28项目录和第一个 Letter 正文，不注入伪造页面/元数据或空实现。三个业务 API 各尝试一次；失败立即停止后续业务，不重试或延长产品单请求超时。输出只有有限字段、状态、字符数、SHA、安全和资源断言，没有完整正文、匿名站点 Cookie 或原错误。

仍需原件/历史 WebView 同输入对照、真实站点认证和其它目标；本模式不声称补齐三方或真实登录。原件不上传 GitHub，不使用已清除的起点 Cookie，也不读取其它本地正文。

## 镜像身份与重复命令

前两轮历史输入：Native成功任务38111391710、测试快照ccbdad907fb803ca4f75a38d182cb86009372dc9、JAR9b9d0c9808968c0390022156a5fcbfcc03f595ed228f124afbb06e830cd7dd84；其失败身份不改写。

下一轮切至本修正自己的已独立接受Native38115254938、快照ec6af77a25a1d048e1751bdf983d3a1894d629ff、准确f5c9 JAR，AMD64镜像75bfc0e1c63cb2e8e2e8d85cdbc31d9b4cb831a24e3bd3cefc09a5d57a20156d。脚本与wrapper两处强SHA守卫同步固定f5c9，不使用latest。维护脚本仍是后续独立checkout，不拿输入镜像绿灯接受新探针或在本模式重建/上传registry。

本机独立下载该任务reader-release-jar artifact11693213191，285,671,807B / SHA f5c9da58803d361849d3cf4e946a3b952081a0a770baf536ec1b6a18e0d6262b，设为只读；与保留9b旧下载副本逐条重算完整JAR及所有条目字节。两者均1570条目，仅Vue前端变化：同名1条改动、各35个哈希名资源删/增；1122个class、90个依赖以及全部非前端共同条目内容不变，包括worker d8b2674c5f28cf31d9003479c7e5035c9d4ebefc0286c0d8a61c4d9672a565f5。[独立字节差异](evidence/opds-frontend-product-byte-delta-2026-10-11.json)，SHAc89c5ad23a93b0aa2397d462f23063fd1967b6aecbe279e4faad8dda57b8e5f6。这里只本机重算JAR，不冒称下载/重算完整原生镜像或已取得新网络业务结果。

复用 `.github/workflows/browser-image.yml` 的显式 `gutenberg-fixed-book` 模式；普通 Full 和起点 metadata 模式不变。仅 GitHub 托管原生 runner、2 CPU / 2 GiB / 256 PID / 零swap、UID10001、cap-drop ALL、no-new-privileges、零宿主端口和私网拒绝。重新校验下载/载入/运行的准确归档、镜像与 JAR 身份，观察实际 worker/browser 进程；有成功外壳但缺书名/作者/目录/正文、资源失败、会话清理失败或未观察浏览器均判失败。

```powershell
gh workflow run browser-image.yml --repo warpdotsys/reader-dev --ref ci/full-reader-20260926 -f public_metadata_native_run=38115254938 -f public_metadata_revision=ec6af77a25a1d048e1751bdf983d3a1894d629ff -f public_metadata_capture=gutenberg-fixed-book -f native_arch=amd64
```

执行条件是指定原生artifact未过期。要重现前两次历史失败，使用其各自准确源码提交及旧输入，而不是给当前f5c9守卫塞旧9b。若原生版本再改变，脚本准确SHA守卫必须先经审阅和实际字节/产物核验更新，不能换入不明latest或修改期望掩盖业务失败。

## 已知问题和回退

### 首轮真实失败与独立观察

[实站任务38113464255](https://github.com/warpdotsys/reader-dev/actions/runs/38113464255)，源d6b7d6a43a6719f6fd51ac85bfe6ab724fd9d989，只执行一个 `/getBookInfo`：HTTP200 / isSuccess=true / 空errorMsg / data对象 / nameMatches=true，但authorMatches=false，按 MissingRealMetadata 判红；目录和首章均未执行，不计作三项通过。生成会话已撤销、Cookie索引0→0、容器已移除。5.425秒，实际worker1 / driver1 / browser7，峰值851,341,312 B / PID180，2CPU/2GiB/256PID，配置和实际swap0，无hard/OOM/PID触限。报告预算scope是最多三项的预定范围，实际执行数以 businessApiCalls 为准。

下载/载入/运行身份均是ccbd快照、9b9d JAR、镜像aa3b64d75108514eb9d1a1ee49dc97f80ba42e447aa7de008b6c5f634f47f8fc，输入Native38111391710/8b25。七个其它条件作业跳过，不计本轮真实执行。九JSON的精确集合/散列、源任务和托管runner、身份、完整有限响应、预算和清理已独立核对。[失败回执](evidence/gutenberg-first-independent-failure-2026-10-11.json)，SHA6b634318a8034cd8356435ffa8dae8dc03371f4f95c3d86d81d3f1c517fadcb7；[原观察](evidence/gutenberg-first-raw-observation-2026-10-11.json)与[原预算](evidence/gutenberg-first-raw-budget-2026-10-11.json)不改写。

### 基于结构证据修正来源规则，不改产品

后续本机直接取样仍为相同468,370B/SHA页面：h3只有25字符且不含Shelley；唯一以 `by ` 起始的h2包含作者。原探针把副标题h3当作者，已改为明确CSS `@CSS:h2:matches(^by\s)@text`，不注入作者名或删除作者校验。原CI没有保存网页字节，不能追补它与本机页面的完整相等证据或认定这是原轮唯一引擎根因。

新增两项真实JVM解析回归，生成标题/h3副标题/h2作者和缺作者页面：正确选择作者，缺失仍空；所在测试类18项全部执行、零失败/错误/跳过。[本机原XML](evidence/gutenberg-author-rule-local-2026-10-11.xml)，SHA01362dfec4e0300b60494cbe63c32beace9c0b0277ebc8976c663ba51b603a7e。Python专用守卫10项、完整637项 / 29.156秒 / 一个既有Windows跳过通过。仅维护来源规则及测试，未改变产品；旧失败保留，新托管实际结果另验，不借d6普通绿灯接受该修正。

### 第二轮实际取得详情与28目录，整体仍失败

[任务38115252638](https://github.com/warpdotsys/reader-dev/actions/runs/38115252638)，源7739ab8516dd60b872ff3a0227cb18e19b8b1948。实际 `/getBookInfo` 和 `/getChapterList` 各一次；两者均HTTP200 / isSuccess=true / 空errorMsg / 完整三字段包络，书名与作者匹配，目录28项，但fixedBookChapterUrlsValid=false，UnexpectedRealToc判红；首章没有执行。9.924秒，worker1 / driver1 / browser7，峰942,919,680B / PID184，无预算触限/OOM，配置和实际swap0，Cookie0→0、会话撤销和容器移除成功。

[独立失败回执](evidence/gutenberg-second-independent-failure-2026-10-11.json)，SHA87838046f2cb53041d098967c8c8359198c7ca7a503fafe74bb5ce6a9ea4a6b9；[原观察](evidence/gutenberg-second-raw-observation-2026-10-11.json)、[原预算](evidence/gutenberg-second-raw-budget-2026-10-11.json)。同样逐项核验九份有限JSON、源任务和精确8b输入镜像、运行JAR、托管runner、清理及预算。没有保存原章节URL值，不能将本机/源码推断追补为该轮实际相对URL观测，更不能认定已证明唯一引擎根因。原失败不改写。

### 目录契约修正候选，不改Reader接口或原来源URL

可读 `BookChapterList` 将所选Jsoup Element交给AnalyzeRule，原规则保留相对href；`BookChapter.url` 也以原相对值序列化，真正请求用 `getAbsoluteURL()` 加baseUrl解析。初始探针要求所有原 `url` 都以完整BOOK开头，这个契约不正确。候选按固定BOOK解析原始URL，仍严格要求28个letter1–4/chap01–24锚点及顺序、索引0–27、非空标题、非卷项、准确baseUrl、原WebView选项、所有解析结果仍属于唯一固定书籍；外站、其它书、缺选项、重复/错序/错误锚点、缺默认字段及控制符均拒绝，不重写原TOC或删除安全检查。

新增六项Python守卫，专用16项通过；完整643项 / 27.879秒 / 零失败 / 一个既有Windows跳过。输出只有有限布尔/计数和完整目录对象的**规范JSON摘要**，不输出原标题、URL列表或正文；摘要不是原始HTTP字节相等证明。

切到本修正自己的准确f5c9产物后，新增第17项强SHA回归，确认探针与wrapper同步固定新JAR且不接受旧9b；最终完整644项 / 28.547秒 / 零失败 / 一个既有Windows跳过。镜像更新依据是7739自己的四条托管终态、原生/重导入实际运行及本机完整JAR条目比较，不调整业务/资源/TLS/私网期望以掩盖失败。新探针实站与其自身普通CI仍需新的准确源码任务。

新增JVM真实解析回归必须与实际所选Element和chapter绑定一致。首轮生成夹具误将整个Document交给 `@href`，结果为空；修正Element后，第二轮又发现测试错误要求绝对URL保留逗号后的空格，而legacy `paramPattern`实际消费了它。两次均20项/一项失败，分别保留[首轮原XML](evidence/gutenberg-toc-rule-local-first-failure-2026-10-11.xml)、[次轮原XML](evidence/gutenberg-toc-rule-local-second-failure-2026-10-11.xml)。修正的是夹具与已有契约断言：原序列化相对URL/选项保持字节，实际解析绝对URL逗号紧接JSON，选项语义仍为唯一webView=true；不改生产解析器或把空结果算成功。修后的真实JVM和新实站结果另验。

修后的JVM已实际20项 / 零失败、错误、跳过，新增两项确实执行；正文用实际chapter绑定，仅取生成HTML中所选letter1段落，不串入另一章节。[原通过XML](evidence/gutenberg-toc-rule-local-passed-2026-10-11.xml)，SHAeaa2387935daf0913ba6e04cdd6b6f0c1547eab388a89731e1644b5bf486d498。它证明本机解析契约，不代替第三轮实站；两份旧失败XML及第二轮目录失败均保持不变。

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

### 第三轮新产物实际详情、完整目录和首章已独立接受

[实站任务38117042668](https://github.com/warpdotsys/reader-dev/actions/runs/38117042668)，源cc17b0ceb964632221168ce1c996c162ab348696，实际托管作业114403888620 / ubuntu-24.04。输入仍明确区分：7739自己的Native38115254938 / ec6af7快照 / f5c9 JAR / 75bfc0 AMD64镜像；维护脚本checkout是后续cc17，不将源镜像或cc17普通CI身份混写。

`/getBookInfo`、`/getChapterList`、`/getBookContent` 各一次，均HTTP200 / isSuccess=true / 空errorMsg / 完整isSuccess/errorMsg/data包络。详情真实书名作者匹配；目录完整28项，实际28个原相对URL、固定base、预期letter1–4/chap01–24锚点和顺序、索引、非空标题、非卷及WebView选项均通过；原序列化URL没有重写。真实首章6822字符，首Letter标记存在、不是资源URL/空实现，没有script标签或U+FFFD替换字符。

目录规范JSON SHA61af49ad646781649f796440dec95dbed27308ca1c7d119b0bc92c287f9450c7；实际首章UTF-8字符串SHAea86a20b840b56b45f9870836bbf8f02ac0a9051f7a2819a6291fe32640a5366。只保留有限计数/摘要，不输出完整目录标题、URL列表、正文、Cookie、凭据或原错误。API入口尝试数不等于目标站点HTTP请求数；本轮未测目标全请求头/请求总数，也没有原件/历史WebView正文对照或前端实际打开此网络书。

20.043秒，实际worker1 / driver1 / browser7；2CPU / 2GiB / 256PID，配置和观察swap0，峰958,705,664B / PID184，无hard/OOM/PID触限。UID10001、私网拒绝与TLS校验保留，源规则完整回读匹配，Cookie索引0→0，生成会话撤销且容器移除；无真实凭据、本地正文、Captcha交互、禁止站内搜索、生产改动或registry发布。

[独立通过回执](evidence/gutenberg-third-independent-success-2026-10-11.json)，SHA8ffe6fa7285e43fd0ae8264eca54b266c463ef3ad9f8fd35affcd54a2679347e；[原观察](evidence/gutenberg-third-raw-observation-2026-10-11.json)、[原预算](evidence/gutenberg-third-raw-budget-2026-10-11.json)。独立核对九份JSON的精确集合/摘要、自己的源任务/托管runner、源Native回执及下载/载入/运行身份、完整有限三项响应、实际浏览器观察、零swap预算和清理。不是把GHA绿色外壳当业务成功，也不借这份实站回执接受cc17自己的普通四条CI或后续源码。

站点页面和上游网络仍是变量；这一固定匿名子集通过不能消除之前起点、认证、三方和资源压力失败。未证明前两轮失败的唯一引擎根因，不通过关TLS、放开私网、改超时/预算或处理验证码取绿灯。原件/历史WebView同输入及网络书前端旅程仍需后续实证。

此前起点、认证头、合并资源预算超时等失败仍保留；本次不发版、不合并或部署。回退只撤回新增显式模式和自有探针，旧起点模式及其失败记录保留；仅清理本轮自有容器和一次性 storage，不动生产或用户工作树。

随后 cc17 自己四条普通托管及 Native 六作业已收齐并独立接受，仍不把三项 API 成功追写为 UI 成功。[新的强制真实界面旅程候选、准确输入和预算范围](GUTENBERG-UI-JOURNEY-2026-10-11.md)。其源码与实测属于后续提交，不能借 cc17 普通绿灯接受。

后续真实UI第一轮业务完成但退出图单色空白，完整画面拒绝；第二轮在相同准确cc17产品中保留原安全/超时/预算，27.456秒实际注册登录入架、读章/翻章/刷新/返回/退出及画面护栏通过，两张无正文图片经主代理实际查看。它是另外一轮有自己源代码和ZIP身份的固定匿名子集，不改写本节API-only结果或以前失败。[两轮结果、图片和已知问题](GUTENBERG-UI-JOURNEY-2026-10-11.md)。
