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

后续诊断补了固定 worker 错误类别白名单：只识别可读 JVM 包装中的已知类别，未知类、附加 URL／Cookie／正文全部不识别、不回显。27 项本机元数据检查通过；真实分类专项尚未复跑，不能回填首轮未观测的类别。

### 同一源码的原生契约失败：不能当作已通过发版验收

`c2679e7b` 的[完整镜像工作流 37430408844](https://github.com/warpdotsys/reader-dev/actions/runs/37430408844)终态成功，但[原生演练 37430408602](https://github.com/warpdotsys/reader-dev/actions/runs/37430408602)在共享 JAR 的默认引擎契约阶段失败；后续原生、重载及导入作业跳过，不能计作成功。两条结果分别保留，不用完整镜像绿灯抵消发布红灯，也不先归因为偶发。

下载的实际 Camoufox XML 为 15 项／0 跳过／1 失败／0 错误；失败的是 `generatedClientNavigationReturnsTheFinalDocumentWithoutReplayingPost`，8.671 秒，通用 `Error`，不足以区分初始导航中断、快照竞态或其他故障。XML SHA-256 为 `4d675d9721c0c9867807ec581cdbf78374e159ac430d89ad269eeef3c1118cd8`；原严格校验实际以 exit 1 拒绝。[失败来源及边界](evidence/camoufox-navigation-failure-c267-2026-10-06.json)。

新增可读 worker 诊断只在异常发生时向 stderr 发出固定操作和类别标签，NDJSON 的原错误类及公开 ReturnData 不变；JVM 测试异常时补生成跳转次数／POST 次数。该契约加强为 12 个独立生成命名空间，每次原 POST 必须恰好一次，首次失败仍立即失败，绝不是失败重试。不会记录异常原文、URL、Cookie、正文或请求内容，不重放 goto／POST／脚本，不吞掉 transport／关闭页面故障，不改变期限／私网防护／成功断言。错误文字匹配仅是诊断提示，不是重试授权或唯一根因证明。相关导航行为可参考 [Playwright 官方导航测试](https://github.com/microsoft/playwright/blob/main/tests/page/page-goto.spec.ts)，但该参考不能代替本项目实际失败定位。

本机全量 Python 171 项，170 执行／1 Windows 环境跳过，零失败／错误；新增 6 项诊断检查确认异常身份／原协议类保留、不泄漏凭据、未知分类保持未知、不重试。它们是无浏览器夹具，不是真实默认引擎验收。新的诊断需要自己的 GitHub 托管真实测试，不改写旧红灯；本机未构建或执行新的 JAR，生产未变。

## 可重复命令

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
