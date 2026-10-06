# 内置浏览器的匿名起点详情验收

## 范围和当前状态

本机仍缺 `uidmap`，不能用宿主 Chrome 或 root 运行冒充最终 UID 10001 的 Camoufox 验收。另建一个明确的托管消费模式：只加载已成功原生演练的完整镜像，不重新构建、替换 JAR、推 registry 或部署。探针源码提交和被测镜像内的修订分开记录，不把旧镜像写成新源码构建。

已完成可读验收入口和两轮实际托管匿名详情：元数据均失败。初版清理断言误读了 legacy 的 setter 顺序，纠正后的独立复跑已验证生成 Cookie 会话失效与容器移除，解析失败仍判红。不能由安全检查、清理通过或成功外壳宣布起点已修好。目标仍包括真实认证、原 JAR / 历史远程 / 默认 Camoufox 的同条件三方、关键 Vue 3 业务和生产用户可见验收。

## 实际验收内容

- 输入限制为同仓库已成功的 `Native release artifact rehearsal`、精确 40 位镜像修订和原生架构。使用现有共享 JAR / 归档 SHA / Docker config / UID / renderer 的守卫，实际加载后再次核对镜像内 JAR。
- Reader 及浏览器仍在同一镜像内，UID 10001；新生成存储和账号，只有容器内部回环监听，没有发布 HTTP 端口。容器允许访问公开站点，但产品的私网拒绝保持开启，未设公网 DNS 替代或修改宿主代理。
- 2 CPU / 2 GiB / 256 PID / 零 swap，权限收紧；采集真实 cgroup 计数。预算只约束 Reader 容器，不把 runner 的镜像下载、Docker daemon 或硬盘用量算成该预算。归档在 GitHub runner 消费，不下载多 GiB 到用户电脑。
- 仅一次 `/getBookInfo`，使用已观测的新详情选择器；不调用搜索、目录、正文、真实账号或 Cookie 导入接口。要求 HTTP、严格布尔 `isSuccess`、`errorMsg` 和期望书名/作者/公开封面一起匹配；成功外壳但空元数据明确失败。
- 只保留字段存在性/匹配布尔值、错误长度/固定原因/域名指纹、请求耗时和进程类别计数；不保存原错误、跳转 URL、HTML、元数据值、正文、匿名站点 Cookie 或 Reader 日志。Reader API 跳转不跟随，响应限 64 KiB。
- 清除生成空间 Cookie，按 legacy 的退出 `isSuccess=true / NEED_LOGIN / 空 errorMsg` 和随后受保护接口 `isSuccess=false / NEED_LOGIN` 两个不同契约核对 Cookie 会话失效。不是全设备令牌撤销。finally 移除本次新容器及其进程；只上传小报告，存储目录和镜像不上传为该探针输出。

## 后续诊断：有限等待动态元数据

旧两轮探针只传 `webView=true`，并未用书源脚本等待动态字段。因此它们证明空元数据，但不能排除返回时机过早；上一轮 Promise 修复也不能自动关闭这一缺口。

新增固定 `--wait-dom`：最多 8 秒、每 100 毫秒只检查原来三个元数据选择器是否已有非空字段。无论字段就绪或达到期限，都返回页面自身的 `document.documentElement.outerHTML`；不插入期望书名／作者，不读写 Cookie、不额外 fetch／导航／重放，也不操作验证码。期限后仍为空则继续由原有严格元数据门禁判红。直接运行不带该标志仍保留旧 `DOMContentLoaded` 基线；托管消费脚本明确带此标志，报告记录 `sourceScriptMode`、预算和“不合成元数据”，不混写两个模式。

本机 Python 162 项中 161 执行／1 个 Windows 符号链接环境跳过，零失败／错误；其中元数据安全／生命周期与新增 JS 驱动共 24 项。Node 实际执行从 Python 源码提取的同一脚本，8 个生成 DOM／时钟检查无跳过，覆盖立即就绪、延迟字段、每个缺失字段、空文本／封面和准确期限。这些是语言／控制夹具，不是真实 Camoufox、起点认证或正文证明。发布结构 57 项通过，未改产品 worker、默认私网拒绝、UID 或零 swap 配置。

新的托管专项必须消费已完成的精确镜像修订，实际结果另记。不能用旧无脚本失败、新生成搜索成功或本机 8 项 JS 来宣称起点已修好。先前实际失败报告原样保留。回滚仅去掉消费脚本的 `--wait-dom` 并保留基线报告；无需改产品或用户数据。

### 第一次有限等待专项仍失败，错误类别尚未确认

探针提交 `c2679e7b69217bd7d869028fa86907c59a6926dd` 的[专项 37430404938](https://github.com/warpdotsys/reader-dev/actions/runs/37430404938)已实际终态失败。只消费已成功演练 `37428262965` 的原镜像修订 `53bf73d680045744ece5db080b0710bb9004a228`，不是新探针的产品构建；实际加载／运行 JAR `12334f82...`、镜像 ID `sha256:e268ce3b...` 与自己的输入一致。

一次详情在 6.175 秒返回 HTTP 200、严格 `isSuccess=false`、非空错误（仅保存长度 70）和 `data=null`，期望字段不匹配。它不同于旧两次成功外壳／空对象；不能写成“等满 8 秒仍无 DOM”，也不能仅由错误长度或较短耗时推断状态丢失、导航错误或唯一根因。实际观察 worker 1／driver 1／browser 类最高 7，UID 10001，私网拒绝、零外部端口保持开启。峰值 `847728640 B`（约 808 MiB）、PID 191，2 CPU／2 GiB／256 PID／零 swap，触限／OOM 0。Cookie 列表前后 0、两个退出契约与自有容器移除均通过；不证明全设备 token 撤销。

3,106 B 小制品的八份 JSON 已下载、逐文件重算哈希，并严格核对身份／范围／预算／清理；失败判红。[完整脱敏结果及未确认的错误类别](evidence/public-metadata-domwait-hosted-first-2026-10-06.json)保留，不保存原始错误、HTML、站点 Cookie 或正文，不借其他普通 CI 绿灯抵消专项红灯。

后续诊断补了固定 worker 错误类别白名单：只识别可读 JVM 包装中的已知类别，未知类、附加 URL／Cookie／正文全部不识别、不回显。27 项本机元数据检查通过；独立分类专项的实际结果见下文，不能回填首轮未观测的类别。

### 同一源码的原生契约失败：不能当作已通过发版验收

`c2679e7b` 的[完整镜像工作流 37430408844](https://github.com/warpdotsys/reader-dev/actions/runs/37430408844)终态成功，但[原生演练 37430408602](https://github.com/warpdotsys/reader-dev/actions/runs/37430408602)在共享 JAR 的默认引擎契约阶段失败；后续原生、重载及导入作业跳过，不能计作成功。两条结果分别保留，不用完整镜像绿灯抵消发布红灯，也不先归因为偶发。

下载的实际 Camoufox XML 为 15 项／0 跳过／1 失败／0 错误；失败的是 `generatedClientNavigationReturnsTheFinalDocumentWithoutReplayingPost`，8.671 秒，通用 `Error`，不足以区分初始导航中断、快照竞态或其他故障。XML SHA-256 为 `4d675d9721c0c9867807ec581cdbf78374e159ac430d89ad269eeef3c1118cd8`；原严格校验实际以 exit 1 拒绝。[失败来源及边界](evidence/camoufox-navigation-failure-c267-2026-10-06.json)。

新增可读 worker 诊断只在异常发生时向 stderr 发出固定操作和类别标签，NDJSON 的原错误类及公开 ReturnData 不变；JVM 测试异常时补生成跳转次数／POST 次数。该契约加强为 12 个独立生成命名空间，每次原 POST 必须恰好一次，首次失败仍立即失败，绝不是失败重试。不会记录异常原文、URL、Cookie、正文或请求内容，不重放 goto／POST／脚本，不吞掉 transport／关闭页面故障，不改变期限／私网防护／成功断言。错误文字匹配仅是诊断提示，不是重试授权或唯一根因证明。相关导航行为可参考 [Playwright 官方导航测试](https://github.com/microsoft/playwright/blob/main/tests/page/page-goto.spec.ts)，但该参考不能代替本项目实际失败定位。

本机全量 Python 171 项，170 执行／1 Windows 环境跳过，零失败／错误；新增 6 项诊断检查确认异常身份／原协议类保留、不泄漏凭据、未知分类保持未知、不重试。它们是无浏览器夹具，不是真实默认引擎验收。新的诊断需要自己的 GitHub 托管真实测试，不改写旧红灯；本机未构建或执行新的 JAR，生产未变。

## 返回快照结构诊断：不读取或保存页面原文

前三次有限等待观测没有记录字段非空布尔或返回页面结构，不能据“不匹配”认定字段为空或一定是验证码。现保留原 `#bookName@text`、作者和封面规则，新增仅用于生成诊断书源的 `intro` 规则：实际 Reader Rhino／Jsoup 解析**已经返回的同一快照**，输出书名／作者／封面选择器存在性、正文文本非空、已知公开书名／作者出现在 body 文本中、安全提示短语及固定验证码容器存在性，共 8 个布尔字段。脚本内容不执行，额外网络请求为 0，原 DOM 等待不改动；真实正文接口仍不调用。

`intro` 只承载本次固定诊断 JSON，并非书籍简介、伪造书名或实际认证结果。Python 只接受精确八键、布尔值、无重复键和 1 KiB 内有效 UTF-8；其他内容不持久化或回显。报告额外记录 `nameHasText`／`authorHasText`／`coverHasText`，不记录实际值。原严格书名／作者／封面门禁不放宽；诊断 JSON 缺失或格式错误即使元数据匹配也不能判整次探针通过，finally 清理不变。

安全短语、容器选择器是返回快照的结构提示：可能隐藏、是普通提示，或站点换了实现。不是“可见验证码”“已认证”或唯一根因证明，不自动操作／绕过验证。生成正常页、提示页、空页、只含 script 文字页和隐藏容器通过真实 JVM `AnalyzeRule` 与 `htmlFormat` 路径的 5 项实际测试，无跳过；31 项 Python 探针检查、全量 Python 176 项中 175 执行／1 Windows 环境跳过，以及 57 项发布结构／制品守卫通过。测试账号／缓存限制在临时目录，未运行原 JAR 或访问真实 Cookie。它们不是实站验收，新托管专项仍须记录探针提交与实际被测镜像修订，不回填旧报告。

回滚只去掉诊断 `intro` 规则及其完整性门禁／配套检查，保留原元数据规则、有限等待和已存在的失败报告；不改产品 worker、用户数据或生产。

### 新专项实际返回脚本状态丢失，未取得页面快照

诊断提交 `6664b63597e5c3d400f65c888a7ed23de438c792` 的[专项 37440328741](https://github.com/warpdotsys/reader-dev/actions/runs/37440328741)已实际终态失败；只消费此前 `c89d6623`／快照 `5eeeb4d5` 的原字节镜像，不把探针提交当作被测镜像修订。八份小 JSON 已独立核对并逐文件重算 SHA-256：实际 JAR `e589818d...`、image ID `sha256:7dea2494...`、UID 10001、私网拒绝、桥接／零发布端口均一致。

这一次仅一个详情请求在 5.426 秒返回 HTTP 200／`isSuccess=false`／`data=null`，严格固定白名单确实记录 `SourceScriptStateLost`，不是按错误长度 70 推断。`pageDiagnostics=null` 表示尚未取得返回快照；`nameHasText=false` 等由无 data 得到，不证明实际页面字段为空。此次可定位到脚本状态丢失这一类别，但尚不能唯一认定导航、脚本干预或具体可见验证码；也不能回填更早的 70 字符错误。没有改产品 worker、重放脚本／POST、吞掉错误或降低元数据门禁。

worker／driver 各 1，browser 类最高 7；真实 cgroup 峰值 811,593,728 B（约 774 MiB）、PID 189，2 CPU／2 GiB／256 PID／配置及实际零 swap、触限／OOM 0，原资源守卫带 `require_no_swap=true` 通过。Cookie 列表前后 0，退出与随后受保护请求、生成 Cookie 会话失效及自有容器移除通过；不是全设备令牌撤销，也没有真实账号／Cookie／目录／正文。3,203 B 制品的[实际身份、观测、八份文件及限制](evidence/public-metadata-page-structure-hosted-2026-10-06.json)保留。其他普通 CI 绿灯不能抵消这一次实站红灯；用户打开的浏览器当前未在工具库存中返回，不能宣称其验证码或正文状态已复核。

### 诊断源码自身的正常托管验收已完成

`6664b635`／被测合并快照 `a5638bea` 的 Java/Kotlin、Vue 3、完整镜像和[原生演练 37440292003](https://github.com/warpdotsys/reader-dev/actions/runs/37440292003)四条正常工作流均终态成功。两份下载的真实 Camoufox XML 各 16 项无跳过／失败／错误；共享 JAR 的真实 JVM 页面诊断 XML 为 5 项无跳过／失败／错误，原脚本在实际 Rhino／Jsoup／`htmlFormat` 路径通过，不是只跑 Python 伪造值。

7,621 B 发布导入制品的 20 份 JSON 已逐文件重算散列并核对共同实际运行 JAR `8d232e90...`、版本／修订对象、原生及重载 image ID、生成基线与四份异步 Reader API；原预算守卫均通过。原生／重载最高峰值 850,243,584 B（约 810.9 MiB）、PID 198，触限／OOM／报告时实际 swap 0，仍允许 1 GiB swap，不能混作零 swap 专项。[实际终态、XML 和消费记录](evidence/public-metadata-diagnostics-normal-hosted-2026-10-06.json)。这个正常结果不消除前述实站 `SourceScriptStateLost`，也不证明真实认证／三方／生产或本机最终镜像运行。

## 可重复命令

### 用无浏览器脚本的同镜像快照基线区分问题

新专项记录 `SourceScriptStateLost` 后，不能假定继续增加等待或重放脚本能修复。新增显式 `public_metadata_capture` 选择输入：`bounded-dom` 保留原来的 8 秒浏览器 Promise，`snapshot-only` 只传 `webView=true`，不传 `webJs`，从正常快照做同一组 JVM 结构诊断。每次仍是新账号／新容器／一次详情，两个模式独立记录并有独立并发组；不是失败重试或把失败请求改成成功。产品 worker、原元数据 CSS、UID／私网防护／预算／清理不变；基线空字段也继续判红，不能关闭认证三方缺口。

模式在下载／加载／启动前严格限定，经过环境变量和引用参数传给 Shell，不把输入作为命令执行。消费脚本旧三参数调用仍默认 `bounded-dom`；第四参数只接受两个固定值。Python 记录 `pageCaptureMode` 与原 `sourceScriptMode`，方便核对究竟有没有浏览器脚本。工作流 `choice` 输入依据 [GitHub 官方语法](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onworkflow_dispatchinputs)，本增量没有给任意 URL、Cookie、脚本或故障重试开放入口。

本机 34 项探针检查、全量 Python 179 项中 178 实际／1 个 Windows 环境跳过、57 项发布守卫和 Bash 语法检查通过。这些是请求参数／流程夹具，不是真实快照结果。新的快照专项待自己的托管执行；必须消费与前次相同的已验收镜像原字节，不将探针新源码当作被测产品修订。

```powershell
gh workflow run browser-image.yml --repo warpdotsys/reader-dev `
  --ref ci/full-reader-20260926 -f native_arch=amd64 `
  -f public_metadata_native_run=37436316345 `
  -f public_metadata_revision=5eeeb4d597836695f27044c91360fbee8aceee3f `
  -f public_metadata_capture=snapshot-only
```

上述输入对应已验收的实际原镜像，不可填任意源码 SHA。单独专项不会构建新镜像、部署或使用真实 Cookie。回滚新增选择输入及第四参数即可恢复旧有限等待入口；旧失败记录必须保留。

### 固定分类的独立专项实际结果

分类提交 `1cf36e7b` 的[专项 37432570576](https://github.com/warpdotsys/reader-dev/actions/runs/37432570576)仍只消费旧已验收 `53bf73d6` 原镜像，不是新提交的产品镜像。此次 12.472 秒，HTTP 200／`isSuccess=true`／空 `errorMsg`／data 对象，但书名、作者和公开封面均不匹配，`workerErrorCategory=null`，门禁仍以 exit 1 失败。它不是上一轮的 worker 异常，不能由本轮空类别回填上一轮的 70 字符错误，更不能以测试未报异常宣称起点恢复。

八份小 JSON／解压散列、两次镜像身份及实际 JAR／修订／零发布端口逐项核对；原资源校验器以 `require_no_swap=true` 实际通过。峰值 868,945,920 B（约 829 MiB）、PID 190，2 CPU／2 GiB／256 PID／零 swap，触限和 OOM 为 0。Cookie 列表前后 0、真实生成会话退出／随后受保护请求、容器移除均通过；未使用真实会话或请求章节。[3,115 B 制品的实际文件和限制](evidence/public-metadata-classified-hosted-2026-10-06.json)。

`1cf36e7b` 的 Java/Kotlin、Vue 3、完整镜像与原生演练四条正常工作流／10 实际作业最终全部成功；5 个互斥可选作业跳过，不计作通过。新共享 JAR 与完整镜像的真实 Camoufox XML 各 15 项／0 跳过／0 失败／0 错误。加强的 12 独立跳转样本分别 36.627／40.191 秒，旧故障这次未复现，不能追认旧失败为成功或声称诊断增量修复了根因。

发布导入器的 7,635 B 制品／20 个 JSON 已独立核对：两架构原生及重载实际 JAR 均为 `046eb858...`、修订 `b5ad4cfc`，版本对象／实际加载 image ID 匹配，四份异步 GET／POST／Cookie 撤销及预算由原严格验证器通过；生成基线和四请求数量也核对。原生／重载短测最高约 809.1 MiB，触限／OOM／实际 swap 0，但仍允许 1 GiB swap，不能混写为起点专项的零 swap 配置。固定 stderr 诊断在已完成的生成作业中实际观测到 `initialNavigation / TimeoutError / unclassified`，属于预期的生成超时，不是旧连续跳转失败的根因分类；未保存原作业日志。[自己的终态、实际 XML／JSON／散列和边界](evidence/camoufox-navigation-diagnostics-hosted-2026-10-06.json)。

此次候选没有导航恢复、异常吞掉、请求／脚本重放或源码层根因修复；真实起点依旧红灯，认证／正文／原件三方／生产仍未验收。结果段落待有意义源码增量再提交，不为追写结果重复触发 docs-only 全量构建。

在已提交的候选分支上，复用已核实的镜像修订而不是探针提交 SHA：

```powershell
gh workflow run browser-image.yml --repo warpdotsys/reader-dev --ref ci/full-reader-20260926 -f native_arch=amd64 -f public_metadata_native_run=37428262965 -f public_metadata_revision=53bf73d680045744ece5db080b0710bb9004a228
```

该模式与原件检查、旧远程、离线长测及通常镜像集成互斥，单独的并发组不会取消其他验收。不能填真实账号、Cookie、任意 URL 或正文参数；未来输入制品过期时须选新的已验收演练并重新核对其准确修订，不能静默换镜像。

## 本机检查、已知限制和回滚

### 纠正夹具后的独立托管复验

探针提交 `929d518d6f90e80ac1f6c87446e0cbd59a7bca18` 的[专项 37419445770](https://github.com/warpdotsys/reader-dev/actions/runs/37419445770)再次只消费同一原镜像/JAR（仍为修订 `a82f0894...`，不是探针提交的重建镜像）。GitHub 原生 AMD64 runner，加载/运行身份与前轮相同，UID 10001、私网拒绝开启，实际观测到 worker 1 / driver 1 / browser 类进程最高 7。入场的 Python 126 项全部实际执行，0 跳过/失败/错误，2.204 秒；其他五个互斥任务明确跳过，不能算通过。

新的真实详情为 6.706 秒、HTTP 200、`isSuccess=true`、空 `errorMsg`，data 为对象，但期望书名/作者/封面仍不匹配，专项正确保持红灯；未请求目录或正文。Cookie 列表前后 0；实际退出观测为 `true / NEED_LOGIN / 空 errorMsg`，随后受保护接口为 `false / NEED_LOGIN / 非空 errorMsg`，生成 Cookie 会话失效已实证，未声称全设备令牌撤销。峰值 861,028,352 B（约 821 MiB）、PID 186；2 CPU / 2 GiB / 256 PID / 零 swap，触限/OOM/PID 最大计数及实际 swap 为 0。自有容器实际移除，3,058 B 制品的八份小 JSON 全部核对/复算解压文件散列。

[准确输入修订、终态、两个真实鉴权观测、预算、文件散列和边界](evidence/public-metadata-hosted-corrected-2026-10-06.json)。首轮红灯和夹具误判原样保留；本轮只关闭夹具清理误判，不关闭真实起点解析/认证问题。没有用普通 CI 绿灯抵消专项失败，也没有改产品源码、发版、部署、安装 uidmap 或读取新浏览器会话。后续只在有意义的源码增量时一起提交新的验收记录，不为追写结果反复触发 docs-only 全量构建。

### 首轮真实托管失败已保留

探针提交 `12fc067c26498d3d981f209482c75096decd3059` 的[专项 37418653036](https://github.com/warpdotsys/reader-dev/actions/runs/37418653036)在 GitHub 托管 AMD64 runner 上实际完成镜像加载与一次默认引擎请求；使用旧已验收演练 `37339021759` 的原镜像修订 `a82f08945a50c1520170fb6e8b3fb3c7a0b296fb`，不是新探针提交的产品构建。镜像 ID `sha256:229a96a1...`、共享/运行 JAR `0491a832...`、UID 10001 和两次身份守卫匹配；观测到 worker 1 / driver 1 / browser 类进程最高 7。

真实请求 5.521 秒，HTTP 200、严格布尔 `isSuccess=true`、空 `errorMsg`、data 为对象，但期望书名/作者不匹配且无封面，明确失败。产品私网拒绝未放宽，没有专用公网 DNS 文件，也没有真实会话或正文请求。内存峰值 826,904,576 B（约 789 MiB）、PID 峰值 201，2 CPU / 2 GiB / 256 PID / 零 swap；原始触限/OOM/PID 最大计数均为 0。

初版探针把退出和退出后受保护接口都误写为 `isSuccess=false / NEED_LOGIN`。核对可读 `ReturnData` 后确认：退出的 `setErrorMsg(...).setData(NEED_LOGIN)` 会由后一个 setter 恢复 `true / 空 errorMsg`，受保护接口相反顺序才是 `false`。只纠正验收夹具的两个明确契约，不修改产品返回、不放宽元数据断言；并增加两次脱敏鉴权观测以独立复验。首轮 `cleanupCategory` 不能当作产品退出未执行的证据，纠正后仍须实际托管验证，不能仅凭源码推断清理通过。

Cookie 列表前后为 0，最后新容器实际移除。下载的是 3,014 B 的脱敏制品，八份 JSON 已逐项核对并复算解压文件散列；远程 ZIP digest 只记录 GitHub 声明，没有冒充本机重算原 ZIP。[首轮完整原结果、身份、预算和夹具限制](evidence/public-metadata-hosted-first-2026-10-06.json)保留。匿名空元数据仍未唯一归因；不能借夹具修正把此次真实解析失败改写成绿灯。

新增 19 项无网络安全/生命周期检查实际通过，覆盖成功外壳拒绝、严布尔/错误契约、非公开封面、原始字段不回显、64 KiB 上限、拒绝跳转和正文接口、UID/私网设置、正常/失败/传输异常后的生成会话清理。它们使用生成数据，不是 Camoufox 真实运行证明。现有正式发布安全静态检查及 24 项测试通过；本机全量 Python 126 项，其中 125 实际执行 / 1 环境跳过、0 失败/错误，不能拿跳过证明 Linux 隔离。Shell 语法和工作流 YAML/九个输入解析通过；真正托管结果另行补记。

一次公开无凭据详情不能验证登录接受、章节权限、所有动态资源、长期并发、原 JAR/旧远程等价、Vue 3 真实用户或生产。失败需保留脱敏结果，不能补空实现、放宽公网防护、绕过验证码或宣称唯一根因。

回滚仅撤销 `browser-image.yml` 的新可选模式及两个探针脚本、配套测试，不涉及产品类、镜像内容或数据迁移。正常集成/离线长测入口保留；不要 reset 用户工作区，也不覆盖原 JAR/现有三个工程。
