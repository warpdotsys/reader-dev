# Production deployment

## 当前状态（2026-10-13）

以[完整交付状态](../../docs/CURRENT-DELIVERY-STATUS-2026-10-13.md)及[正式发布就绪核查](../../docs/DELIVERY-READINESS-2026-10-13.md)为准：准确548a整镜像已完成获准单书八场景和固定公开书HTTPS三方，较早的大书失败证据仍保留；c703自己的托管验收也已接受。两者镜像身份不混用，不宣称整个Goal已完成或可直接部署。正式流水线已经等待原生AMD64/ARM64构建、独立重载和发布器导入；它不是下文历史阶段的AMD64-only路径。

生产SSH四个Secret已在GitHub的production环境中列名确认，仓库级列表没有这些名称不代表缺失。仅核查名称，未读取私钥或Secret值，不证明凭据有效或部署成功。用户选定的v6.1.0已同步后端、备用CLI、两套前端及锁文件根包，依赖版本不改。[同名发布说明](../../docs/releases/v6.1.0.md)及[本候选精准证据](../../docs/evidence/candidate-f6e61bf3-hosted-acceptance-2026-10-13.json)现已记录f6e61bf3自己的四条托管验收、真实下载JAR及有限生成UI通过；准确新镜像的实站／单书／原件三方及真实账号仍未闭合。最新正式版v6.0.45（master）的发布排序门禁保留，未打标签或部署，不移借c703/548a结果。

下文带日期的验收、失败与部署禁止状态按当时范围保留，不覆盖最新独立结果。

## 历史候选记录与部署要求

当前原版 EPUB 测试已加强文档就绪判据，但授权大书首次解压仍超时，不能部署；新测试提交必须独立经过托管流水线。[实测、已知问题与回退](../../docs/EPUB-FRAME-READINESS-2026-10-13.md)。

最新 `e31` 四托管全验收，但授权大书刷新仍未通过；8KiB实验已撤回，不得作为修复部署。[准确产物、失败证据与回退](../../docs/EPUB-ALLOCATION-TRIAL-2026-10-13.md)。

当前仍是候选验收阶段，不得以旧托管绿灯部署新源码；[导入生命周期候选、已知大 EPUB 刷新超时及回退](../../docs/BOOKSHELF-IMPORT-LIFECYCLE-2026-10-13.md)。

`reader-pro-restored` runs beside the existing Reader container and binds only
to `127.0.0.1:18088`. The host Nginx terminates TLS for `read.medwarp.cn`.

Required server-side files that are deliberately not committed:

- `.env`: authentication and deployment configuration, mode 600;
- `storage/`: imported Reader data, owned by UID/GID 10001;
- `logs/`: application logs, owned by UID/GID 10001;
- release image: pulled by digest from the registry; production never rebuilds
  from a JAR on the host.

## 正式发布

2026-10-13准确新产物验收：1dd自己的四托管 / Native六作业 / 发布器小报告均通过，A031 JAR两次clean同字节且仅Vue变化；实际授权大EPUB在原账号登录前置下两轮原版刷新仍expanding超时，六 / 八，不发版部署。MainThread实为Node测试驱动，纠正后的驱动约100MiB，不把旧混合组作为浏览器或驱动独因。所有自建实例 / 副本已清，原件和用户数据保留。[实际失败、完整构建证据及下一项](../../docs/EPUB-TERMINAL-RELEASE-ACCEPTANCE-2026-10-13.md)。

2026-10-13源码候选：EPUB可复用清理保留，新增终态release供卸载 / 作废加载结果清整本资源引用；338前端及类型 / Vite、153管线生成测试通过。不把组件卸载等同于硬刷新全部生命周期，自己的托管和准确JAR / 真书待验，Y六 / 八失败仍在，继续不发版部署。[候选证据与回退](../../docs/EPUB-TERMINAL-RELEASE-CANDIDATE-2026-10-13.md)。

2026-10-13认证兼容后续：原JAR生成账号加载后真实既有登录、错误密码、注销撤销和新会话通过；随后授权大EPUB原版刷新再现expanding超时，阅读六 / 八。V/W双轮成功不等于稳定，保持原门槛并继续阻止发版部署；自有实例和副本已清，原件及用户数据保留。[认证证据、阅读失败与下一项](../../docs/ORIGINAL-ACCOUNT-RELOGIN-2026-10-13.md)。

2026-10-13完整页复测：同准确JAR / 授权单书 / 原限额，V与W两轮真实八场景均通过，无产品修改或代码注入。复制后加载也通过，不能称找到旧超时原因；软内存回收 / 系统CPU压力已观测，稳定与三方等门槛仍未齐，继续不发版部署。实例及仅本轮副本已清，原件和用户数据未改。[双轮结果、风险与剩余交付](../../docs/EPUB-FULL-READER-UI-PRESSURE-2026-10-13.md)。

2026-10-12 真实业务前置定位：U10通过实际预览导入、保存书架、1,611章目录及授权章节接口，再359ms下载 / 1.664秒解压 / 全CRC一致；同资源和超时门槛，没有产品修改。不是完整Vue页面或真实八场景通过，T四/八仍在，继续禁止发版部署；临时实例及输入副本已停止、清理，原件和用户数据保留。[实测范围与剩余问题](../../docs/EPUB-FIREFOX-DECODE-DIAGNOSTIC-2026-10-12.md#真实业务前置后的流式对照)。

2026-10-12 原流式路径定位：受测源码AST逐字提取并转译，在真实文件API最小页824ms读全、锁释放/独占buffer、2.811秒回复/全CRC一致；不是完整打包Reader/Vue，也未做真实入架/章节预处理，T四/八仍未修复，不发版。实例与精确三输入副本已清，原件和用户数据保留。[流式源码对照与下一检查](../../docs/EPUB-FIREFOX-DECODE-DIAGNOSTIC-2026-10-12.md)。

2026-10-12 实际传输定位：生成身份的真实同源文件API准确长度、无压缩分块、命名空间匹配，arrayBuffer诊断读取1.252秒/解压2.085秒/全CRC一致；没有生产流式读取、入架预处理或完整Vue，不替代真实T四/八失败，不发版部署。临时实例与副本已清，原件和用户数据保留。[真实API对照及下一项](../../docs/EPUB-FIREFOX-DECODE-DIAGNOSTIC-2026-10-12.md)。

2026-10-12 继续定位：准确JAR空闲后端加入最小Firefox诊断后，原堆/8线程池/同限额仍5.517秒完成全资源、CRC一致；没有Reader/Vue完整页面或实际文件API，不替代T四/八失败。实例和精确输入副本已清，原文件/用户数据不改，不发版。[空闲后端对照与下一检查](../../docs/EPUB-FIREFOX-DECODE-DIAGNOSTIC-2026-10-12.md)。

2026-10-12 后续定位：独立Firefox最小生成页的同一实际Worker/唯一授权EPUB在同限额内1.913秒返回全部资源、独立CRC一致；没有Java或Reader页面，不替代真实T四/八失败或允许发版。框架启动失败保留，诊断实例及精确输入副本已清，原件和用户数据未改。[子阶段范围与下一项诊断](../../docs/EPUB-FIREFOX-DECODE-DIAGNOSTIC-2026-10-12.md)。

2026-10-12 最新：7bf自己的四托管、Native六作业及32个publisher小报告已复验，准确308d JAR下载SHA与双次clean相同，仅Vue变化。但真实授权单书T仍四/八，首次原样在expanding超时、后续刷新未执行；ready和JS200仅证明本次启动链通过，不能关闭阅读故障。Node计数仅子阶段诊断，原件/授权源/58用户报告保留，临时输入副本已清；不发版或切换生产。[已知问题、正式产物、诊断范围与回退](../../docs/EPUB-READY-BEFORE-TRANSFER-2026-10-12.md)。

2026-10-12 后续候选：Worker 正确 ready 后才转移输入，取消/超时/重复确认不能再发送；15 秒总期限及所有 ZIP、下载、隔离限制不改。37 定向和 332 全前端/零跳过、类型/Vite 已执行，但自己的正式产物及同书八场景待验，不把此前 789 的绿灯当本候选验收，不作生产切换。[时序候选与回退](../../docs/EPUB-READY-BEFORE-TRANSFER-2026-10-12.md)。

2026-10-12 最新：789 的准确正式 JAR 已独立核 SHA 与自己的双次 clean 重建；Java、Vue、Full 的生成消费器通过，但授权单书 S 仅四/八，首次原样 Worker starting 超时，原样刷新未执行。没有真实阅读修复 / RSS 改善结论，不进入生产发布；本轮隔离进程和两份输入副本已清理，原件、授权来源及用户 58 报告保留。Full 的缓存使用确有观察，207 秒构建 / 58.2 秒发送不证明稳定提速或严格总 60 秒限。[正式产物、失败复测、已知风险和回退](../../docs/EPUB-FORMAL-ARTIFACT-ACCEPTANCE-2026-10-12.md)。下列为捕获时历史，不覆盖新结果。

2026-10-12：a80接受而真实单书R仍六/八，刷新解压超时，不能发版。403 Full/Java/Vue接受、Native固定下载HTTP504拒绝；缓存导出152.8秒，不把timeout60s当已证总限或提速。新有界下载缓冲已326前端/真实输入三头字节一致，固定URL最多3次重试已7生成及755本机Python（1原平台跳过）执行；版本、完整性、预算及Reader超时不改，自己的正式构建/八场景待验，不作生产切换依据。[最新原失败、候选与回退](../../docs/EPUB-DOWNLOAD-BUFFER-CANDIDATE-2026-10-12.md)、[8/8线程配置历史](../../docs/VERTX-POOL-CANDIDATE-2026-10-12.md)。

2026-10-07 数值线程修复 `09163efe` 自己的四条托管工作流／原生六作业、两套各 19 项真实契约及小身份／预算报告已独立验收。[完整已知风险与回滚](../../docs/CAMOUFOX-NUMERICAL-THREADS-2026-10-07.md)仍保留实站失败和本机旧 worker 的 2 GiB／256 PID 触限。之后的有限[内存分类采集](../../docs/BROWSER-MEMORY-CATEGORIES-2026-10-07.md)也完成自己的 `594cec9c` 托管验收，5 份实际分类／原资源守卫通过；仅补观测，不扣除文件页、不清宿主缓存、不升预算取绿灯。没有正式发版、安装服务器新浏览器或切换生产产物。

正式版本通过 `.github/workflows/release.yml` 发布。全部验收完成后，准备与标签同名的
`docs/releases/<tag>.md`，并从 `legacy` 上创建高于当时最新正式Release的稳定SemVer标签。
触发前必须同步提升 `build.gradle.kts`、`web/package.json` 与
`web-vue3/package.json` 的版本号；流水线拒绝版本不一致、非稳定 SemVer、非 `legacy`
祖先提交，或不高于当时最新 GitHub 正式 Release 的标签。

发布前还必须在仓库设置中配置以下值，工作流不会猜测仓库名或凭据：

- Variables：`GHCR_IMAGE`（格式 `ghcr.io/namespace/image`）与
  `DOCKERHUB_IMAGE`（格式 `namespace/image`）；
- Secrets：`DOCKERHUB_USERNAME`、`DOCKERHUB_PASSWORD`；
- 已有生产连接 Secrets：`PROD_SSH_HOST`、`PROD_SSH_USER`、`PROD_SSH_KEY`、
  `PROD_SSH_KNOWN_HOSTS`。其中 known_hosts 必须是 `cdn.medwarp.cn` 的已验证主机键。

`PROD_SSH_HOST` 必须精确为 `cdn.medwarp.cn`，工作流会拒绝其他目标。若 Docker Hub
镜像为私有镜像，或服务器可能被匿名拉取限流，部署账户必须已在服务器完成可用的
`docker login`；工作流在停止旧容器前先拉取并按 digest 验证新镜像，认证或限流失败不会
中断正在运行的服务。生产服务器不会接收 Docker Hub token。

SSH 只使用仓库 Secret 中的已验证 `known_hosts`，并显式启用
`diffie-hellman-group14-sha256` 与 `IdentitiesOnly=yes`；这与该主机已验证的兼容
协商保持一致。每份正式发布说明还必须有 `## 已知问题…` 章节，缺失时不会开始构建。

`requirements-camoufox.lock` 已按 Ubuntu 22.04 / Python 3.10 为 38 个固定依赖记录
52 个兼容 amd64/arm64 wheel 的 PyPI SHA-256，且本机校验了依赖闭合和 pip 锁文件解析。
这尚不等于镜像安装成功：正式流水线会强制 `pip --require-hashes`，并从
`base-images.lock` 读取已核验的 Temurin 与 Playwright manifest digest，再以这些
digest 构建；发布时不会重新从浮动 tag 解析 digest。Dockerfile 不接受浮动基础镜像
默认值；固定的 Playwright Python 镜像已含 Python、浏览器及其系统依赖，构建过程
不再访问曾返回 5xx 的 Ubuntu apt 快照源。更新任一锁文件必须作为可审查的发布链路变更。

流水线会：

1. 在 GitHub 托管 runner 上重新运行 Vue 3、Java/Kotlin 测试，构建可校验 JAR；
2. 构建包含 Camoufox 的唯一完整镜像并在镜像内进行健康和浏览器运行时烟测；
   烟测服务必须仅监听回环，临时目录限额 256 MiB，并在推送镜像之前检查
   2 GiB / 256 PIDs / 2 CPU 的 cgroup 限额及 OOM/PIDs 事件；
3. 将同一镜像按版本标签推送到 GHCR 和 Docker Hub，读取并比对两个 registry digest；
4. 创建正式 GitHub Release，附 JAR、SHA-256 和镜像 digest 清单；
5. 由 `cdn.medwarp.cn` 拉取 Docker Hub 的 digest 引用，不从本地 tar 或服务器源码构建；
   激活前要求可用空间至少为 `3 × storage + 2 GiB`，保存 storage、配置和旧镜像回滚快照，
   再做健康与 `read.medwarp.cn` 公网烟测。公网探测同时校验 `/assets/reader-release.json`
   中的版本和构建提交；失败会先确认容器正使用本次 digest，再自动恢复已验证的旧镜像与数据。

可在创建标签前执行以下本地静态检查；它不访问 registry 或生产服务器：

```bash
node scripts/check-release-pipeline.mjs
```

该检查也由普通 Java/Kotlin CI 和正式发布输入检查执行。正式产物目录增加
`BROWSER_RESOURCE_BUDGET.json`，记录该次完整镜像烟测的原生架构、资源峰值及
限额事件；它不证明生产长期吞吐。当前`release.yml`复用`release-native.yml`，要求
GitHub托管的原生AMD64及ARM64构建、两侧独立重载、正式发布器导入均通过后，才推送
两个已经受测的镜像并创建多架构manifest，不在发布器内重建。普通Native演练不登录registry，
因此已有六作业成功不能宣称正式manifest已发布或生产已验收。

以下命令只用于已有 digest 镜像的紧急手动恢复，不作为正常发版流程：

```bash
cd /opt/reader-pro-restored
READER_IMAGE='registry/image@sha256:...' docker compose config --quiet
READER_IMAGE='registry/image@sha256:...' docker compose up -d --no-build
curl -fsS http://127.0.0.1:18088/
```

## 唯一的完整镜像

`Dockerfile` 将 Reader JAR、Python 解释器、固定版本的 Camoufox `0.5.6` /
Playwright `1.62.0`、Firefox 依赖和经 SHA-256 校验的
`v152.0.4-beta.30` 浏览器放进**同一个容器**。运行时设置
`PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`、
`READER_APP_WEBVIEWRENDERER=camoufox`，并使用
`/usr/bin/python3` 启动 worker；不要求 Docker 宿主机安装
Chrome、Firefox 或独立 WebView 容器。只构建一个完整版本，不再构建或发布轻量版。
远程 WebView 配置暂留作回滚。生产 Compose 使用 Docker 内建 `init: true` 回收
浏览器子进程；这不是独立的 WebView 服务。

手动以 `docker run` 启动完整镜像时也必须带 `--init`；使用其他容器运行器时，
须提供等效的孤儿进程回收。不要直接沿用上游旧镜像的启动示例并省略这一项。
同一 v6.1.0 镜像的[本机 A/B 验证](../../docs/LOCAL-V6.1.0-2026-10-15.md)
显示：未回收的测试 PID 1 累积 44 个僵尸后触及 256 PID 限额；补齐回收后，
原 4 请求、原限额下完整生成书源测试通过。这是运行环境要求，不是生产故障已复现。

在 JDK 11 环境下，从仓库根目录构建唯一产物：

```bash
PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 npm ci --prefix web-vue3
npm test --prefix web-vue3
npm run build --prefix web-vue3
PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 ./gradlew -PreaderWebUi=vue3 clean test bootJar --no-daemon
# 用当前版本 JAR 创建隔离镜像上下文，再传入 --build-arg READER_JAR=<jar filename>
```

版本化 Reader JAR 包含 Java 端 renderer 和 Python worker 脚本；Camoufox
二进制及其 Python 运行时在镜像内，不在 JAR 内。构建后的 JAR 复制到镜像上下文只是
构建输入，不应提交到 Git。GitHub 托管 runner 的
`.github/workflows/browser-image.yml` 会先安装固定浏览器并执行 GET、POST、脚本、
Cookie 用户命名空间和 `sourceRegex` 资源捕获测试，再构建镜像并在隔离数据目录下运行
合成书源搜索及 Cookie 回归；生产部署仍需另行验证和发布。

历史短测快照：Camoufox 仍处于候选接入阶段。[207711f6 的 GitHub 托管作业](https://github.com/warpdotsys/reader-dev/actions/runs/36513001239)
已通过 Linux 真实 Camoufox 合约、无 apt 单镜像构建及受限容器内四账号合成书源烟测；
未压缩镜像约 3.93 GiB，短时 cgroup 内存峰值约 737 MiB。该结果不是生产镜像发布或
真实站点容量承诺。[7daacc72 的镜像作业](https://github.com/warpdotsys/reader-dev/actions/runs/36513674140)
还证实容器内存在支持中文的文泉驿正黑字体。上述大小与短测资源只属于这些历史制品。

2026-10-06 当前候选 `08cccfca`／受测镜像 `d5c00919...` 已完成自身双架构
构建、重载／导入、真实 Camoufox 18 项与生成数据门禁，详见[本轮验收](../../docs/CAMOUFOX-SOURCE-DOCUMENT-2026-10-06.md)。
同一制品的 AMD64／ARM64 离线四账号长测实际持续 1800／1813 秒，2 CPU／2 GiB／PID 256、
零 swap 下峰值约 930／931 MiB，触限／OOM 为 0，三类故障后恢复通过。
静止内存仍增约 68／67 MiB，突发最长请求约 23.79／20.04 秒；不是无泄漏或容量承诺，
也没有证明真实代理／书源、原 JAR 与历史远程服务的完整三方兼容或生产网络隔离。
详见[准确长测范围和风险](../../docs/BROWSER-SOAK-2026-10-06.md)。同镜像起点匿名解析仍失败，
真实认证及生产切换未完成。该受测版本的 Dockerfile／Compose 并未默认选择 Vue 3；如显式选择
`READER_APP_WEBUI=vue3`，须使用已经包入 `web-vue3` 的制品并单独验收界面与回滚。

后续源码已将完整镜像／Compose 默认入口改为 Vue 3，保留同名变量 `READER_APP_WEBUI=vue2`
回退；普通 JAR 默认值不变。新的静态字节门禁贯穿原生构建、传输重载和发布导入，
`de7c0c7e` 自身默认入口验收保留；后续手机布局修复 `e5cc2ba8` 也已完成自己的
双架构构建／重载／导入和普通完整镜像验收。新增长分组断言仅本机通过，
真实起点匿名解析仍失败，真实认证和生产未验收；不得拿旧绿灯代替新制品。
[手机布局准确证据与回退](../../docs/SHELF-MOBILE-CONTROLS-2026-10-07.md)。
[最新默认入口候选、已知故障与回退](../../docs/DEFAULT-VUE3-IMAGE-2026-10-06.md)。
HTTP 和 SOCKS4/5 上游代理由本地出口代理支持，并有固定 IP、认证和凭据隔离单元测试；
尚未用真实第三方代理或真实书源验证兼容率。

GitHub runner [36131976893](https://github.com/warpdotsys/reader-dev/actions/runs/36131976893)
记录的是旧 Chromium 实现的功能基线：重定向内网拦截、JS 子资源、分块 SSE、GET/POST 与
Cookie 隔离，以及单个 Reader 镜像内的合成书源烟测。Camoufox 的对应候选验证见上述历史作业及当前候选验收；二者都不等于原生产远程 WebView 三方差分。应用层出口代理不能替代主机或容器防火墙；公网不可信
书源部署仍应在网络层拦截云元数据、loopback、RFC1918 与 IPv6 ULA。workflow 的 loopback
烟测显式启用了 `READER_BROWSER_ALLOW_PRIVATE_NETWORKS=true`，只用于固定合成夹具，
不应照搬到不可信书源可写入的生产实例。若后续兼容性回归要求回退，可在同一完整镜像中设置
`READER_APP_WEBVIEWRENDERER=remote`，但只有原远程 WebView 服务实际可用时才可采用。
不要在没有数据备份与正式发布验证的情况下直接替换
生产容器或挂载生产 `storage/data`。
