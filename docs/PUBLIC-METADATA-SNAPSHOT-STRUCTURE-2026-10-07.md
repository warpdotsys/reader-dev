# 起点返回快照的有限结构诊断

## 目的与范围

起点匿名详情仍是已知红灯。上一轮的八个布尔值全部 false，只能证明返回快照没有匹配的选择器和 body 文本，不能证明原字符串为零长度、可见验证码、认证成功或唯一根因。本增量增加可选 `bounded-dom-details`，不改产品类、默认书源规则或已有生成详情门禁。

该模式仍在已验收的完整 Reader 镜像内运行，UID 10001、2 CPU / 2 GiB / 256 PID、零 swap、私网拒绝、零发布端口。生成账号保存书源后读回精确规则，仅一次 `/getBookInfo`、同一固定起点详情 URL、原 8 秒只读 DOM 等待。不请求目录或章节，不导入真实凭据、复制浏览器会话、操作验证码或增加重试。

新增 intro 规则仅解析浏览器已经返回的字符串，产生原八布尔值和九个有限结构值：

- `rawEmpty`、`rawWhitespaceOnly`：原字符串零长度或非零长度纯空白。
- `rawHtmlTagHint`、`rawHeadTagHint`、`rawBodyTagHint`：原字符串的标签文本提示，不等于真实解析出的标签；注释或脚本也可能产生提示。
- `headTitleElementPresent`、`headScriptElementPresent`、`bodyHasChildElements`、`frameElementPresent`：Jsoup 解析后的有限结构，不是可见性、导航完成或认证结论。

Jsoup 会自动生成 html/head/body，因此不能通过这些自动节点判定原响应包含标签。不会保存 HTML、标题文本、URL 参数、Cookie、元数据原值、任意脚本或原错误。结构结果只允许两组精确键和严格布尔值；未知键、重复键、缺失、类型错误、非法 UTF-8 和超过 1 KiB 的诊断均拒绝。缺少新诊断不能拿旧格式补 false，更不能让空元数据通过。

默认规则 SHA-256 仍为 `603a86357f45c3b850ec3c8cce2fe98266053aa1276fdf7178fbd5a4fe61166a`；默认等待脚本仍为 `045f640185166e2d9415e09b1e2d574ba45cf6119345ba180f4fe6e5a9bf4b04`。新结构规则为 `064b02c65bb5450c981d171b9de7eb58a7b287ac9bdfb055e90fe8c456b3521f`，实际报告记录其散列。

## 生成控制与验收边界

已经独立消费 `5a4c6b9e17146cfda8c826e1ed78aebe3ede1248` 的普通托管结果，而不是首次错误参数的专项或旧原件：

- [Native 37617038662](https://github.com/warpdotsys/reader-dev/actions/runs/37617038662)：六个实际作业全部成功，受测合并修订 `95525b8c8bde54f74f7df85f04489e9a181ef6dd`。
- [完整镜像 37617038875](https://github.com/warpdotsys/reader-dev/actions/runs/37617038875)：实际通过；同一源提交的 Java 和 Vue 工作流也已终态成功。
- 下载六份小制品，不下载本机大镜像；两份默认引擎 XML 各 20 项无跳过、两份 helper 各 3 项、24 个 publisher JSON 实际核对。
- 五份默认 UI / 异步 / 原预算和五份真实生成详情回执由原严格守卫独立接受。峰值 867,950,592 B / PID 203，内存 max / OOM / PID max / 实际 swap 为 0，但普通短测配置允许 1 GiB swap，不能混写为匿名专项的零 swap。
- Native JAR `c3b7e572ba351e224778087d3c0afac35986cd152e0abd53d7b53a5fe9511f35`，Full JAR `70b4bc97d4c0dad87a0684af21402821557d56be03a9fe7c1e024d97d9e6c488`；不能混作原 JAR、服务器三方的 221 JAR 或此前匿名专项。
- 本机独立回执 `build/hosted-snapshot-control-5a4c6b9e-20261007-a/independent-publisher-acceptance.json`，SHA-256 `b47f9966b65603f586b9257312d356b31d62a57283002cc174f3bfb3fb0e4587`。58 个用户报告的 SHA 均未改变。

这些是默认生成链路的控制证据，不是新结构规则的真实浏览器执行或起点成功证据。新规则有生成 Python 和真实 Rhino / Jsoup / WebBook 测试；JVM 的预填 `infoHtml` 不经浏览器，不能冒充实际匿名请求。

本次入场实际结果：探针 Python 48 项通过（新增 12 项），全 Python 299 项中 298 实际通过 / 1 Windows 环境跳过；发布安全检查 49 项通过，Shell 语法、工作流 YAML 与封闭枚举通过。真实 JVM 16 项（新增 7 项）全部实际执行、零跳过 / 失败 / 错误；XML SHA-256 `a2363656145301db31dcf4adf2c2994d3f47f9ec5c79f197d1d122225b8d129e`，本机保留副本。首次测试编译因新断言对可空 intro 直接调用而失败；修正为必须非空的断言后完整重新编译、执行通过，没有修改产品返回或填充空实现。

本机 `uidmap` 已核实来自 Ubuntu 官方 `resolute/main`，版本 `1:4.17.4-2ubuntu3`；`newuidmap/newgidmap` 可用，短命用户命名空间检查通过。未重复安装、改 subuid/subgid 或代理；这不代表最终镜像 UID 10001 运行、既有资源红灯或本机真实登录已经验收。

## 可重复执行与回滚

入场测试：

```powershell
$env:READER_TEST_NODE='C:\Users\chong\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe'
python -m unittest discover -s src/test/python -p public_metadata_probe_test.py
./gradlew.bat --offline --no-daemon --max-workers=2 '-Dorg.gradle.jvmargs=-Xmx512m -XX:ActiveProcessorCount=2 -Dfile.encoding=UTF-8' test --tests com.htmake.reader.utils.PublicMetadataPageDiagnosticTest
```

受测产品与探针 checkout 身份分开；专项只消费上述已完成 Native 的原字节：

```powershell
gh workflow run browser-image.yml --repo warpdotsys/reader-dev --ref ci/full-reader-20260926 -f native_arch=amd64 -f public_metadata_native_run=37617038662 -f public_metadata_revision=95525b8c8bde54f74f7df85f04489e9a181ef6dd -f public_metadata_capture=bounded-dom-details
```

只有精确规则保存读回、元数据、结构、原资源和清理均满足时才允许 exit 0。实际专项结果另记，未执行前不得写成功。本增量自身正常 CI 也必须单独核对，不能用 `5a4c6b9e` 追认。

## 已知问题

### 本次独立托管专项仍失败，未获得结构

探针提交 `7198c7f0fe461fe41d229e50b33d6cea1e0e2cd2` 的[专项 37621716112](https://github.com/warpdotsys/reader-dev/actions/runs/37621716112)终态失败，唯一实际作业 112793349345；六个互斥任务跳过，不计通过。只消费上述 `5a4c6b9e` / `95525b8c` 的原 Native 镜像，不以新探针提交冒充产品修订。

生成书源的精确规则读回通过，但唯一 `/getBookInfo` 在 6.502 秒后返回 HTTP 200 / 严格 `isSuccess=false` / 非空错误 / data null，固定类别 `SourceScriptStateLost`。`pageDiagnostics` 和 `snapshotStructure` 均 null，没有实际执行新 intro 规则的浏览器证据，不能填 false 或称“等满 8 秒仍空”。有限 worker 记录为 `sourceScriptRead / sourceStateMissing / mainFrameNavigationObserved=true / sourceDocumentObservation=unavailable`；这证明读取阶段状态缺失且期间观测过主框架导航，不证明相同/不同文档、验证码、认证、导航就是唯一原因，也不解释上一轮不同响应的空元数据。

九份小 JSON / 两次镜像及运行 JAR 身份已独立核对，原 `require_no_swap=true` 资源守卫通过：峰值 872,595,456 B / PID 186，2 CPU / 2 GiB / 256 PID / 零 swap，触限 / OOM / PID max 为 0。Cookie 列表 0→0，生成会话退出及随后保护接口验证成立，自有容器移除报告 true；没有独立宿主剩余容器计数，不能扩大清理证据。原 58 用户报告未改变。

[完整有限证据与逐文件散列](evidence/public-metadata-snapshot-details-7198c7f0-2026-10-07.json)记录 3,896 B 远程 ZIP 声明、身份、失败、资源与清理；仅重算实际下载解压的文件散列，不冒充本机重算远程 ZIP digest。独立回执 SHA-256 `139f4758b353feed1e267a83cb672ec5ddc981c498df2ac16b1e158079f65cba`。旧红灯保留，不自动重放可能有副作用的书源脚本、不重试同条件、不吞错或放宽门禁。结果文档待下一个有意义的源码增量一起提交，不为纯结果回填重启全量构建。

本结果随[生成详情三方门禁增量](GENERATED-METADATA-THREE-WAY-2026-10-07.md)一起提交，不单独触发纯结果文档构建。该增量的历史两侧已实际观察，不代表真实会话或内置侧已通过。

匿名起点空元数据、真实认证及原 JAR / 旧远程 / 内置的真实会话三方仍未完成；正文、生产和长期高核数容量没有新增接受证据。结构分类最多缩小排查范围，不会自动修复业务、放宽防护或绕过验证码。原红灯和旧 UTF-8 请求差异保留。

回滚仅撤销可选 `bounded-dom-details` 枚举、`--snapshot-details` 规则及配套测试；原 `bounded-dom` / `snapshot-only`、默认八布尔值、8 秒脚本、生成详情门禁和下载修复均保留。没有数据迁移，不删除用户文件或原 JAR，也不重置工作区。

## 2026-10-08：无额外等待脚本的有限结构入口

带等待脚本的实站请求此前在取得快照前失败，不能靠补 false 推断结构；旧 `snapshot-only` 虽取得成功外壳／空元数据，却只有八个选择器／文本布尔值。新显式 `snapshot-only-details` 仅组合既有无源等待脚本请求与既有九项有限结构诊断，以取得缺失的观测，不宣称修复站点或规避挑战。

它映射到 `--snapshot-only-details`，与 `--wait-dom`／`--snapshot-details` 冲突时，在环境检查、容器或账号操作前拒绝。原三模式不变，旧 details 仍要求 bounded wait；新模式不含 `webJs`、源等待预算为 0，**不是禁用网站自身 JavaScript，也不是整个请求超时为 0**。现代书源及精确规则必须保存读回，详情仍严格一次；原 8＋9 布尔值、1 KiB 诊断上限、所有元数据／资源／真实浏览器观察／清理门槛均保留。空元数据即使结构完整仍失败，不保存 HTML、Cookie、原异常或真实元数据值。

新增四项测试先在旧 CLI 实际出现 52 项／4 errors（新参数不存在），完整输出保留；这是新入口缺失的基线，不叫产品回归。实现后 52 项全执行通过；Windows 全套 334 项中 333 通过／1 POSIX 跳过，22.804 秒，Node 原发布契约 82＋固定 DOM／时钟语言用例 8 共 90 全通过，Shell／YAML 语法通过。WSL 首次 52 项中 1 项因缺 Node 失败，未掩盖或放宽；仅用固定参数桥调用现有 Windows Node 后 WSL Python 52 项通过，JS 子用例不是原生 Linux Node或浏览器证明，没有安装额外软件。

基于源码 `ba3ce478...`／被测合并快照 `7e4b9e50...` 的四个普通 hosted workflow 此时均已成功，Native 六实际作业／24 publisher JSON 已独立接受；不是拿这些绿灯称新实站模式已执行。两份真实 Camoufox 各 20 项无跳过，helper 各 3 项；Vue 核心 19＋管理安全 2＋子目录 1 实际执行，TTS 7.614 秒、四张生成图已目视，返回 A 的图处于淡出过渡，不冒充稳定最终弹窗。14 组登录几何核对，普通 JVM 170 中 31 环境跳过不计作浏览器。

Full JAR `29d07c04...` 与两架构共同 Native JAR `61fdc23d...` 来自同一测试快照，但散列不同；各自默认 UI、异步／生成详情、预算及 Native 原生／重载身份分别接受，不能混作同一 JAR或称位级可重复构建。差异唯一原因尚未核验，后续检查归档时间／内容；没有仅凭缺少归档选项就唯一判因。短测 Full 峰值 845,168,640 B／PID 193、Native 最高 921,747,456 B／PID 204，实际 swap 0但普通配置允许 1 GiB，仍非零 swap 长测。没有把两份约 2 GiB 归档下载到本机或冒充本机重算归档散列。

准确字节、旧失败、两个 runtime 身份及局部验收见[机器证据](evidence/public-metadata-snapshot-only-details-2026-10-08.json)。提交新模式后仅消费已通过的 `37718718336` / `7e4b9e509822473c6d98d17c8bb0b922fe28a406` Native 归档，受测 JAR与探针提交分别记录：

```sh
gh workflow run browser-image.yml --repo warpdotsys/reader-dev --ref ci/full-reader-20260926 \
  -f native_arch=amd64 -f public_metadata_native_run=37718718336 \
  -f public_metadata_revision=7e4b9e509822473c6d98d17c8bb0b922fe28a406 \
  -f public_metadata_capture=snapshot-only-details
```

当前新模式实站仍未执行，结果另行独立接受；不重试同条件请求或操作验证码。原件／旧远程真实认证三方、严格 UTF-8 原缺陷、最终长期及生产仍未完成，58 用户报告未变。回滚只撤销这个第四枚举和独立 CLI flag，保留原三模式、规则散列、失败证据和用户数据；不改产品 worker、业务类或代理防护。
