# Reader 代理修复的托管构建与实际 HTTPS 业务核验

日期：2026-10-10。源码 `22f949ccb3befff4d66af793598889b5879b0c45`，托管测试合并快照 `d74875d2ea057500cadedcb66af4c7942e3f8d96`。GitHub API 对比前进一提交、文件差异为零。默认分支仍为 `legacy`，PR56 保持 draft。本轮没有合并、创建 Release、推送 registry、部署或导入生产数据。

后续已将实际API客户端整理为可维护入口，并重新取得8项业务/12项worker本机观察；新托管模式尚待自己的提交验收。5ec父提交四条托管已独立接受，但其业务33项仅为静态守卫。[后续接入和准确边界](PORTABLE-READER-HTTPS-GATE-2026-10-10.md)。下文保留22/a轮捕获时状态，不改写成后续结果。

## 已成功重建：这次提交自己的四条托管任务

不是借用 ec20 的绿灯。实际作业、runner、测试 XML、小产物与阶段身份分别核查；原生六个作业均成功，使用 GitHub `ubuntu-24.04`／`ubuntu-24.04-arm`，不是自托管。

| 实际任务 | 独立接受的范围 | 原始核查回执 |
| --- | --- | --- |
| [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/38049613396) | 525 Linux Python／零跳过；198 JVM／44套件／35个原环境跳过；新增4项代理测试均执行；两次干净 JAR 字节相同 | [Java](evidence/proxy-appdir-hosted-java-22f949cc-2026-10-10.json) |
| [Vue 3](https://github.com/warpdotsys/reader-dev/actions/runs/38049613394) | 296前端检查、20截图守卫、25实际生成 UI 流程／零跳过，11张听书设置截图 | [UI 与 Java](evidence/proxy-appdir-hosted-ui-java-22f949cc-2026-10-10.json) |
| [完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/38049613409) | 24 Cam／23 Chromium／3 helper；生成 Reader API 和默认界面；新增实际打包 worker 的12项 HTTPS 场景／24请求／2证书负例 | [Full](evidence/proxy-appdir-hosted-full-22f949cc-2026-10-10.json) |
| [双原生与重导入](https://github.com/warpdotsys/reader-dev/actions/runs/38049613326) | 六实际作业；amd64／arm64 构建与转移重导入；publisher 28个小 JSON，含四份完整 TLS 原始报告，各自逐字段重验 | [Native](evidence/proxy-appdir-hosted-native-22f949cc-2026-10-10.json) |

新 JAR SHA-256：`fcd28a8d974666f12de6671570f2435a9161789d29f20314c7e1f28bf8f2285f`，285,671,163B／1,570条目。已从当前 shared-JAR 作业产物11669291303下载，并在本机流式重算实际 JAR 字节 SHA，副本设置为只读。下载使用自有 Windows 作业、2逻辑CPU／512MiB限制，约94.66秒，峰值149,045,248B；没有下载巨大 native 镜像归档。下载 ZIP 和 native 大归档的 SHA 仅来自产物声明及阶段交叉核查，未在本机重算，不混同 JAR 的实际字节验证。

普通 Reader 镜像任务最高885,784,576B／PID195；原生普通任务最高873,459,712B／PID197。它们允许 swap、实际使用0。新增 Full 打包 worker HTTPS 的独立容器为2CPU／2GiB／256PID／零swap，实测峰值629,018,624B／PID152，`memory.high=max`；这些托管预算不能冒充下方本机的 high1.5GiB 策略。

主 agent 实际查看了本次 Full 的生成登录和中文阅读截图，两张中文可读、登录控件可见。只接受这两张的目视范围，不宣称整个界面编码或真实生产登录已解决。

## 已从当前实际 JAR 验证：Java 出口到选定代理的业务链

[原始生成观察](evidence/reader-business-https-22f949cc-2026-10-10.json)，SHA-256 `2427acdaaf7e48835bdb73a0f29689017c186a6f242f528c13c4379c0e4b3a28`；[独立接受摘要](evidence/reader-business-https-independent-2026-10-10.json)。这是新 fcd JAR 的真实注册／登录、保存及回读书源、搜索 API，不是 renderer 替身或直接调用 worker 后拼装 ReturnData。只有生成账号、生成正文、生成假认证头；没有原账号密码、起点 Cookie、其他本地书正文或生产数据。

- 按顺序执行8个场景：A同源GET、A跨源POST303、A同源POST307、A重复GET、B同源GET、A跨源POST308、错误域名证书、未信任CA。生成页面脚本实际改写 DOM，搜索解析产生中文书名与作者；保存／回读的规则和代理一致。
- 6个正例均 HTTP200／isSuccess=true、errorMsg为空、data一条；完整14个搜索字段及空值／整数默认值原样保留。2个负例均 HTTP200／isSuccess=false，有真实错误消息，**没有 `data` 字段**；核查器保留这一实际序列化形状，禁止人为补 `data:null`。
- 12个真实目标 HTTP 请求、14条 CONNECT 隧道、178个解析头字段。上游代理每次在确认隧道前，读取仅该自有 JVM 的 PID起始tick、fd/socket inode 与本隔离网络的 TCP 表，核对客户端五元组确属当前 Reader。目标 TCP 源端口与对应隧道逐项匹配，不只凭一个成功布尔值。
- 同源保留假 Authorization／trace，跨源剥离；Host 与实际隧道 authority 一致；303转换为无正文且无 POST 类型的GET；307／308保留39字节原始UTF-8正文与显式application/json媒体类型。这里不是宣称这份表单形状正文是合法JSON对象。`proxy`不泄漏成源站HTTP头。
- 同源Secure Cookie在A后续请求保留，B请求没有A Cookie，B初始书源列表为空；跨源不携带源站Cookie。验证的是两个生成账号的这一有限状态序列，不是全部用户生命周期。
- 负例在TLS阶段分别实际产生 `SSLV3_ALERT_BAD_CERTIFICATE`／`TLSV1_ALERT_UNKNOWN_CA`，关联对应隧道源端口，目标HTTP请求为0。没有修改宿主信任、忽略证书、覆盖 worker 启动函数或开放外网。

本机约52.39秒，UID/GID10001／空组／空能力／no-new-privileges，独立user/net/mnt/pid/ipc/uts命名空间、仅lo，2CPU／2GiB／256PID／零swap、high1.5GiB；root/home/浏览器/输入JAR只读，只有生成临时区可写。峰值1,611,509,760B／PID240；hard max、OOM、PID触限0，但**soft high事件2468**，明确保留，不能称资源压力为0。收尾时内存类别主要为file缓存，不是读取或压缩用户海量数据，也不是峰值RSS的精确拆分。

Java在外层清理前有序退出143；仅结束自有运行时，独立确认PID0、删除该容器且自有清单0。生成凭据和私有日志在该次临时文件系统，不写入回执。授权安装的官方 uidmap 已使用；未改变映射范围或系统代理。

运行时为旧缓存镜像 `sha256:401d895c31a39634bf29654a7fbb29527272d287c8c1219b12794bbe1970bc44`，只读绑定新 fcd JAR；**不是最新整个 native 镜像的本机验收**。本机生成内部程序SHA为 `8b61045c3bafb9ea1c86587bce2997a8a596cdb658e1e061ef2afe7225e226a3`（原运行报告已记录）；控制器 `ae4e6aa567e3bd71df28730ae81f153d561669202f77315c0b9f2c807c31f8fe`、API客户端 `f79e5fe41ec7612fd39cd96c1cfc427f0af60e4e032b9ec1de9def519b18a286` 是本轮收尾重算，不冒称后两项已在运行时捕获。

## 可重复核验与构建

下列核验只重验上述真实生成记录，**不会重新启动浏览器**；33项静态守卫也不等于33次真实渲染。初次32项及557全量通过后，再增加一项四份托管回执的精确字节守卫；最终Windows全量558项Python／33.790秒／失败0／1原POSIX平台跳过。仅为这六份新证据设置Git字节保留属性，避免跨Windows／Linux检出时改变原SHA，不改历史回执或全局Git配置。58份受保护用户报告及原JAR／只读备份重新核对未变。

```sh
python3 scripts/verify-reader-tls-business.py \
  docs/evidence/reader-business-https-22f949cc-2026-10-10.json \
  --jar-sha fcd28a8d974666f12de6671570f2435a9161789d29f20314c7e1f28bf8f2285f \
  --revision 22f949ccb3befff4d66af793598889b5879b0c45
python3 -m unittest discover -s src/test/python -p '*_test.py'
```

构建命令仅用于干净的独立 checkout／GitHub托管环境：

```sh
./gradlew -PreaderWebUi=vue3 clean test bootJar --max-workers=2 --no-build-cache --no-daemon --stacktrace
```

禁止在当前保留了授权书籍、证据和未提交数据的本机 `build` 目录上执行clean。本次新的8场景实际API测试尚未接入托管工作流，工作流中的33项新守卫是静态记录测试，不能把此前托管的worker HTTPS门槛称为托管Java业务链验收。

## 后续补充：当前 fcd 的首轮有限 HTTP 三方

在保存上一阶段5ecbc609提交后，又对同一当前fcd产物运行新n轮，不借用旧574的三方结果。[独立回执](evidence/origin-header-fcd-current-jar-three-way-2026-10-10.json)，SHA `36141b5701e39cdc2c2c08727171aa4cc9b006ac593f69ef69c225c5cf2454e0`；[完整原始观察](evidence/reader-fcd-three-way-raw-2026-10-10.json)，SHA `3863969c7fafb7aa179a41ce12e274240c8b791cb9579daf92168e32e229df40`。

准确b26原件、新fcd JAR的历史远程模式、新fcd的Camoufox模式，实际执行五项基础Cookie／脚本／POST契约，并追加same-get、cross-get、cross-post303、resources四组。12份完整ReturnData逐字相等；实际状态均200／isSuccess=true／errorMsg为空，data字段存在。基础15目标请求、追加25目标请求均来自生成夹具。原件与恢复远程模式的角色排序完整HTTP字段相同，字面请求顺序也相同。

原件与固定历史引擎仍向跨源目标发送生成Authorization／trace；新版Cam保同源、剥跨源，且正确跟随旧侧未跟随的POST303。**当前有限业务结果兼容通过，但严格安全三方相等仍为false**；不是把旧泄漏行为改写为通过或让新引擎回退。固定历史镜像manifest `sha256:88c250043bd33715ca372b749c1dca055c14e4550882caf968bd7af933d53573`的身份已核对，但不能证明它就是原JAR当年线上使用的WebView版本。

约77.148秒；两自有非root运行时同user/net、分离mount/pid/ipc/uts、仅lo，顺序停止历史引擎至PID0后才允许Cam；aggregate2CPU／2GiB／high1.5GiB／256PID／零swap，峰值1,611,759,616B／PID242，soft high5584／hard max与OOM、PID触限0，自有清理0，生成历史PID1回收36个已收养僵尸。此轮client用私有home/cache临时区，**不冒充上一节的只读home HTTPS条件或最新完整镜像**，也没有原件HTTPS或真实站点认证。

核验完成后仅删除这一已停止n轮的两份输入JAR副本358,585,050B；原件、只读备份及四份CI下载重新核对并保留，可从这些字节重建临时副本。不删除任何其他缓存目录、书籍或测试证据。

## 尚未验证、已知问题与回退

- 当前fcd已取得上节自己的有限HTTP三方；原件HTTPS、显式proxy差异、编码扩展及完整业务兼容仍未接受。旧ac0和本次有限子集都不能移植为完整版本结论；不能恢复旧跨源认证泄漏追求相等。
- 原件非ASCII正文编码差异、getBookGroups编码差异保留；不能归一化原版数据来消除失败。
- 当前错误消息只到 `Camoufox 渲染失败 (Error)`，证书原因由生成目标端握手观测确认；不能称已经向用户提供了精准TLS诊断。
- 真实站点登录／购买边界、授权真实书阅读全旅程、WS/WSS头作用域、长时间与冷启动资源、并发／崩溃恢复、最新整个镜像本机验收仍需补齐；OPDS后端仍未实现。
- 默认分支 `legacy`、PR56 draft、生产版本均未变。本轮不发版部署。两个源参数改动为明确有意兼容修正，回退按[源码代理修复说明](SOURCE-PROXY-FORWARDING-2026-10-10.md#可重复核验与回退)做单独revert，不reset用户工作树；测试tmpfs及报告门槛按独立提交回退，保留旧失败和原始JAR对照。

完整项目目标尚未完成。这一回执只接受指定产物、指定生成业务子集和上述资源／清理边界。
