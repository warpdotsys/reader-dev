# 浏览器证书错误提示与自己的托管业务证据

日期：2026-10-10。默认分支仍 `legacy`，PR56 draft；没有发布、registry写入、生产部署、真实凭据/正文导入或全量语言重写。

## 已成功重建：c29自己的真实业务门槛

源码 `c29e682b309214dd53f54891177869473fd043f1`，实际测试快照 `c01fb598dded4c812ffd397b0a34f85275f7f108`，API比较领先一提交、文件差异零。它自己的Java与UI产物已独立接受：586 Linux Python / 零跳过；198 JVM / 35原环境跳过；25实际生成UI / 零跳过。两次干净JAR位级相同，仍为已实际下载重算的fcd SHA，285,671,163B / 1,570条目，不宣称又重算了native大归档。

四条自己的任务已全部终态成功并独立核对，均为GitHub托管ubuntu-24.04 / ubuntu-24.04-arm，不是自托管：

| 实际任务 | 独立接受的范围 | 回执 |
| --- | --- | --- |
| [Java](https://github.com/warpdotsys/reader-dev/actions/runs/38060527987) | 586 Linux Python、198 JVM、双干净复建一致 | [Java](evidence/reader-https-hosted-java-c29e682b-2026-10-10.json) |
| [UI](https://github.com/warpdotsys/reader-dev/actions/runs/38060527995) | 25实际生成UI、零跳过；不是全页面目视或真实账号验收 | [UI / Java](evidence/reader-https-hosted-ui-java-c29e682b-2026-10-10.json) |
| [Full](https://github.com/warpdotsys/reader-dev/actions/runs/38060528134) | 准确镜像worker12项、实际Reader HTTPS8项、普通Reader及默认UI | [Full](evidence/reader-https-hosted-full-c29e682b-2026-10-10.json) / [业务原始观察](evidence/reader-https-hosted-full-raw-c29e682b-2026-10-10.json) |
| [Native](https://github.com/warpdotsys/reader-dev/actions/runs/38060528043) | 六作业；双架构构建/重导入各worker12项+Reader8项，publisher32份小JSON逐字段重验 | [Native](evidence/reader-https-hosted-native-c29e682b-2026-10-10.json) |

[Full任务](https://github.com/warpdotsys/reader-dev/actions/runs/38060528134) 已终态成功并独立核查。旧worker12项之后，新增实际Reader API8项真实执行并保存原始记录，不再只靠33项静态守卫；生成注册/登录、书源保存/回读、搜索/页面脚本、用户Cookie隔离以及两证书负例均执行。12目标HTTP / 14CONNECT / 178头字段，每条上游连接由当前Java PID起始tick/socket inode/五元组实时匹配。

该实际业务容器2CPU / 2GiB / 256PID / 零swap，UID/GID10001、仅lo、空能力、no-new-privileges、只读根/浏览器/JAR、私有生成storage及CA策略；峰值1,033,232,384B / PID194，hard/OOM/PID触限0，high=max，不冒充本机high1.5GiB。Java在外层清理前退出143。普通Reader/UI阶段峰值883,904,512B / PID188，预算允许swap但实际0，不能把两个阶段合成一个预算结论。

Full实际业务原始JSON SHA `991b3be6099cf6ec0ac161d71c80d255659db05fa14cd4d28c167ad45c3b58f6`；独立Full回执SHA `d4e672933aeca8fba58f2947eeca5022bccc420b04e3aa8ab4dbfdb3d2f9ab51`。Native六作业/最终publisher32份JSON现已独立接受，回执SHA `ac26ce6f81c597ee982908948d5bae77e84fcffef2f6379a0cd75d6b135a6dc2`。四份准确业务观察分别为 [amd64构建](evidence/reader-https-hosted-native-amd64-raw-c29e682b-2026-10-10.json)、[amd64重导入](evidence/reader-https-hosted-native-amd64-transfer-raw-c29e682b-2026-10-10.json)、[arm64构建](evidence/reader-https-hosted-native-arm64-raw-c29e682b-2026-10-10.json)、[arm64重导入](evidence/reader-https-hosted-native-arm64-transfer-raw-c29e682b-2026-10-10.json)，各自严格8项 / 12HTTP / 14CONNECT / 178头字段。Native业务阶段最高964,313,088B / PID203，2CPU2GiB零swap；普通阶段最高943,366,144B / PID202，两个阶段不混同。本轮没有下载/重算大归档或把这些托管结果当作最新整镜像本机/真实站点证明。

## 已从源码核查：用户提示丢失的原因

现有worker刻意只返回异常类名，以防Playwright异常文本里的URL/凭据进入日志或ReturnData。Java因此把实际证书域名/签发机构失败都显示为 `Camoufox 渲染失败 (Error)`。c29真实负例依然是这个泛化提示；它们只证明证书拒绝和业务形状，不证明本节候选提示已生效。

Mozilla官方分别说明 `SSL_ERROR_BAD_CERT_DOMAIN` 是证书不适用于请求站点，`SEC_ERROR_UNKNOWN_ISSUER` 是签发者无法建立信任，可能涉及证书链配置；不能仅凭它认定攻击或唯一根因。[官方错误码说明](https://support.mozilla.org/en-US/kb/what-does-your-connection-is-not-secure-mean)。

## 候选修复：可读源码，不改变证书决策

- Python仅对**initialNavigation阶段、Error类、Page.goto错误前缀**识别上述两个有限码。出现在普通URL/任意文本/源脚本错误中的同名字符串不分类；未知/不可打印错误仍保留原行为。
- 内部worker协议只额外传有限 `certificateError`，不传原始异常、URL、正文、Cookie、token或证书文本；诊断也只使用有限标签，不重放请求、不绕过校验、不自动导入CA。
- Java仅当 `error == Error` 且字段严格等于白名单值时，返回固定中文“HTTPS证书域名不匹配”或“HTTPS证书签发机构不受信任”及有限码；未知值保留泛化失败，成功结果不能被单独hint变为失败。父进程网络策略拦截仍优先于worker提示。
- 这是两种错误 `errorMsg` 的**有意用户可见改进**，不是宣称其字面文字与原件一致。HTTP状态、ReturnData字段/isSuccess、省略data的失败形状、成功正文及持久化结构不做改写；这些行为的新实际JAR回归仍待运行。
- 新构建的producer及publisher均要求 `--require-certificate-hints`，真实API两证书负例必须各自返回对应固定消息，同时满足原8项HTTP/头/Java连接/证书拒绝条件。只出现普通Error、消息互换或包含URL/私有文本都不能通过。

旧包络和历史c29报告默认不强求新文字：保留原始观察，不将旧失败改写成新通过。新门槛按当前源码显式启用，不能用旧绿灯接受新提示。

下列命令只重验c29的历史原始报告，不启动浏览器，也不要求后来的固定中文消息：

```sh
python3 scripts/verify-reader-tls-business.py \
  docs/evidence/reader-https-hosted-full-raw-c29e682b-2026-10-10.json --packaged \
  --jar-sha fcd28a8d974666f12de6671570f2435a9161789d29f20314c7e1f28bf8f2285f \
  --worker-sha c3ef2486a4b6ed37e0f3a898518dd4173c42e270c62d49d87e881562c50f4c24 \
  --revision c01fb598dded4c812ffd397b0a34f85275f7f108 --architecture amd64
```

新源码可重复构建仅在干净独立checkout / 托管runner执行：先构建 `web-vue3`，再用JDK11运行 `./gradlew -PreaderWebUi=vue3 clean test bootJar --max-workers=2 --no-build-cache --no-daemon`。不要对当前保留授权书籍/证据的本机build执行clean。

## 本机已成功编译；真实浏览器提示尚未验证

[准确本机回执](evidence/certificate-hints-local-verification-2026-10-10.json) / [实际JVM协议XML](evidence/certificate-hints-local-parent-protocol-2026-10-10.xml)。598 Python / 32.069秒 / 失败0 / 1原Windows平台跳过；86流水线检查 / 零跳过；编译后的Java/worker协议4项 / 零跳过，包括错误码固定中文、未知值/错误类别、成功与网络策略优先级。新增Python6项、业务报告6项、Node2项均为明确生成双桩，不冒充额外真实浏览器执行。worker源码/Gradle资源字节一致，SHA `d8b2674c5f28cf31d9003479c7e5035c9d4ebefc0286c0d8a61c4d9672a565f5`；四文件3.10语法、两个Shell语法通过。

首次测试辅助函数把可空body声明为String，compileTestKotlin失败；改为String?后实际编译/4项运行成功。新增flag后，一项忽略错误的删改测试插入点不完整，先触发“缺少flag”检查；修正到完整命令尾后86项全通过。保留这两次准备失败，不把它们当作生产证书失败或抹成首次通过。本机构建只限制该次JVM为2可用CPU/768MiB、最多2worker，不运行clean，不修改用户构建资料。

候选自己的JAR/实际浏览器/两架构及重导入提示尚未取得证据。下一步是提交源码后使用GitHub托管runner的完整8项业务门槛，检查真实错误前缀及完整ReturnData；若未识别就保留失败修正，不为了绿灯放宽TLS或消息校验。

## 已知问题与回退

原件HTTPS/完整三方及编码扩展、真实站点认证/授权真实书完整旅程、WS/WSS、并发长测/冷启动/崩溃恢复、最新整镜像本机运行仍未齐；原件非ASCII/getBookGroups编码差异、OPDS后端未实现继续保留。证书诊断仅覆盖两个已识别码，不宣称涵盖过期、吊销、中间人或所有网络失败。

回退采用候选提交的独立revert：撤回内部hint字段、固定消息和新flag，保留原始报告及旧8项门槛；不要reset用户工作树，不删除数据，不恢复跨源认证泄漏，不忽略证书。当前原JAR/只读备份及58份受保护报告已在提交前重新核对，字节均未变，备份仍只读。
