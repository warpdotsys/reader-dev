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

## 初次提交前：本机已成功编译；当时尚无真实浏览器提示证据

[准确本机回执](evidence/certificate-hints-local-verification-2026-10-10.json) / [实际JVM协议XML](evidence/certificate-hints-local-parent-protocol-2026-10-10.xml)。598 Python / 32.069秒 / 失败0 / 1原Windows平台跳过；86流水线检查 / 零跳过；编译后的Java/worker协议4项 / 零跳过，包括错误码固定中文、未知值/错误类别、成功与网络策略优先级。新增Python6项、业务报告6项、Node2项均为明确生成双桩，不冒充额外真实浏览器执行。worker源码/Gradle资源字节一致，SHA `d8b2674c5f28cf31d9003479c7e5035c9d4ebefc0286c0d8a61c4d9672a565f5`；四文件3.10语法、两个Shell语法通过。

首次测试辅助函数把可空body声明为String，compileTestKotlin失败；改为String?后实际编译/4项运行成功。新增flag后，一项忽略错误的删改测试插入点不完整，先触发“缺少flag”检查；修正到完整命令尾后86项全通过。保留这两次准备失败，不把它们当作生产证书失败或抹成首次通过。本机构建只限制该次JVM为2可用CPU/768MiB、最多2worker，不运行clean，不修改用户构建资料。

以上是首次提交前的状态；提交后的实际运行及失败边界如下。不能用该初始状态或以前的绿灯接受本轮产物。

## 已从准确恢复JAR验证：fc真实提示生效，验收前缀断言失败

源码 `fc0860f1f9251a9ad2fa25d649e60f8b44068e3c`，测试快照 `65d7cd8144f85cb1316d102d26ca6b0604cfe632`。自己的 [Java任务](https://github.com/warpdotsys/reader-dev/actions/runs/38062537108) 和 [UI任务](https://github.com/warpdotsys/reader-dev/actions/runs/38062537167) 终态成功，独立核对598 Linux Python / 零跳过，44套201 JVM / 35原环境跳过，25实际生成UI / 零跳过。两次干净复建完整字节一致，新JAR SHA `0882fcf42692723381fc20dd746f269df96035c0c2b1c336d0fed027782bdf56`、285,672,007B / 1,570条目，不借用旧fcd身份。[Java回执](evidence/certificate-hints-hosted-java-fc0860f1-2026-10-10.json) SHA `9354631bbf9cd1e4a153d9aefc77c1b8479091f91f19ff8ff4c7944205f4cb04`；[UI/Java回执](evidence/certificate-hints-hosted-ui-java-fc0860f1-2026-10-10.json) SHA `fcbcac57fe9a58c5e9793adae3ec59db6db2a7210bdcd710e838cabf7a56f6cb`。这些不证明生产UI或真实书完整旅程。

[Full任务](https://github.com/warpdotsys/reader-dev/actions/runs/38062537142) 和 [Native任务](https://github.com/warpdotsys/reader-dev/actions/runs/38062537104) **终态失败**，不得标成成功。Native两架构实际执行8个生成Reader业务场景，两个证书负例的真实ReturnData都是HTTP200 / isSuccess=false / 省略data，提示分别为：

```text
java.lang.IllegalStateException: Camoufox HTTPS 证书域名不匹配 (SSL_ERROR_BAD_CERT_DOMAIN)
java.lang.IllegalStateException: Camoufox HTTPS 证书签发机构不受信任 (SEC_ERROR_UNKNOWN_ISSUER)
```

两个负例各自目标HTTP请求为零，并留下对应TLS失败/Java上游连接记录。新生产提示已在自己的准确JAR中真实出现；失败发生在**验收脚本只期待renderer.message、未包含Reader原有Throwable.toString()前缀**。`YueduApi.onHandlerError` 源码也明确保留 `error.toString()`。修复只调整脚本的完整固定wire文字，不修改接口去掉前缀、不做任意strip或后缀匹配、不忽略证书。

保留 [amd64失败日志JSON](evidence/certificate-hints-hosted-native-amd64-failed-raw-fc0860f1-2026-10-10.json) SHA `ac08ee86d873160963250ca8cb094ef1c9640cfc8ab7cecec983aac9dabbff9d`，[arm64失败日志JSON](evidence/certificate-hints-hosted-native-arm64-failed-raw-fc0860f1-2026-10-10.json) SHA `0abb147cb993777001aa6cba643a908a0326688025705055395d2e979e7dff8e`。它们是从对应失败作业日志原样提取的payload，**GitHub已经遮盖Authorization字段，不是未经遮盖的产物报告**。不复原这些星号、不声称用它们独立接受了完整178头字段/8项业务；新增两个记录回归只严格核对准确身份、真实两错误完整文字/形状/零目标HTTP。第一次尝试将遮盖日志投入完整头验证被拒绝，此边界保留。

Full同样停在固定消息断言，后续完整镜像默认UI报告不存在，上传步骤明确失败；Native重导入及publisher作业被跳过。需要用修正后的producer/publisher新任务取得**未遮盖原始产物**并重跑完整门槛，不能把失败作业更名为通过，也不能先拿失败Native任务执行长测。

脚本修正后本机602 Python / 36.825秒 / 零失败 / 1原Windows平台跳过、55业务报告守卫及86流水线守卫通过。4个新增Python用例包括缺失/任意前缀拒绝和两架构真实错误记录回归；记录回归不是新的浏览器执行。生产worker仍为d8 SHA，Java/Kotlin生产源码未再次改变。长测将在新成功准确镜像上进行，保持2CPU / 2GiB / 256PID / 零swap与生成数据隔离。

## 已成功重建：c158自己的完整镜像接受固定wire消息

源码 `c1581a9edc33b1653157207c4dd2cae9a896f63e`，自己的测试快照 `0c5f744cdd0d730b33cc3cf64b5d85c5826c2405`；源码与快照文件差异零，不能借fc或c29任务结论。自己的 [Java任务](https://github.com/warpdotsys/reader-dev/actions/runs/38063801059)、[UI任务](https://github.com/warpdotsys/reader-dev/actions/runs/38063801098)、[Full任务](https://github.com/warpdotsys/reader-dev/actions/runs/38063801151) 均终态成功并独立核对：602 Linux Python / 零跳过、201 JVM / 35原环境跳过、25实际生成UI / 零跳过；自己的双干净复建为0882 SHA / 285,672,007B / 1,570条目，与fc相比生产源码/前端未变化。不声称本机重新下载重算了这个JAR或native大归档。

[Java](evidence/certificate-hints-hosted-java-c1581a9e-2026-10-10.json) SHA `87013f44b87ac5f43661bc45d7a888b0ccab69190559709a2387828a373f8f4d`；[UI/Java](evidence/certificate-hints-hosted-ui-java-c1581a9e-2026-10-10.json) SHA `35ca6f19eaa29f17cc703c3bf45f2d6779b6bafd62a8d89ac047389b1b2406cb`；[Full独立回执](evidence/certificate-hints-hosted-full-c1581a9e-2026-10-10.json) SHA `448a87ae4735d8116a0d4699d764a252a8f6c91ed27a66c7f2ebd2ed0ea18792`。

Full取得 [**未遮盖的实际业务产物**](evidence/certificate-hints-hosted-full-raw-c1581a9e-2026-10-10.json)，SHA `f060d3a95346256925a42130293a4c79c8f269164846f6a5265ecfb55bfddaff`，不是日志payload。它真实执行8场景 / 12HTTP / 14CONNECT / 178头字段、两个生成用户注册登录/Cookie及书源隔离、两个证书负例；独立完整字段守卫和新固定消息门槛均通过。HTTP200 / 失败isSuccess=false / 省略data，两个负例目标HTTP均0；没有放宽证书、伪造ReturnData或补复原遮盖头。原fc失败和遮盖日志完整保留。

该真实HTTPS业务阶段2CPU / 2GiB / 256PID / 零swap、UID10001、只读根/浏览器/JAR及私有CA/storage；实际峰值956,583,936B / PID194，hard/OOM/PID触限0，high=max。Java在外层清理前退出143。普通Reader/UI阶段918,089,728B / PID198，配置允许swap但实际0；这两个阶段不混作同一预算或本机high1.5GiB结论。真实默认UI、Cam24、Chromium23及worker12也在Full各自接受。

此段捕获时Native双架构构建成功，重导入正在运行，最终publisher尚未结束；**不是Native全绿或可发布声明**。30分钟新镜像长测和当前JAR完整原件三方尚未完成。新听书源编辑通知候选另见 [通知归属、会话隔离与验收边界](TTS-EDIT-NOTIFICATION-OWNERSHIP-2026-10-10.md)，不能用c158当前的UI绿灯接受尚未提交的改动。

### 后续终态：Native自己的六作业与32份报告全部接受

[Native任务38063801166](https://github.com/warpdotsys/reader-dev/actions/runs/38063801166) 六个作业现已终态成功，均为GitHub托管runner。独立逐字段核对32份publisher JSON、自己的Cam24和helper3、双架构构建/重导入各自worker12及**实际Reader HTTPS8项**；四份业务报告全部含新固定wire提示并满足12HTTP / 14CONNECT / 178头字段/Java socket归属/两证书HTTP前拒绝/两生成用户隔离。没有登录registry、推镜像、发版或修改生产。

[Native独立回执](evidence/certificate-hints-hosted-native-c1581a9e-2026-10-10.json) SHA `cf6a78ce070ffa8f5edd9924485b635e9440fda81ed37c0ba1a07e0ba9f9d27e`。业务原始报告：[amd64构建](evidence/certificate-hints-hosted-native-amd64-raw-c1581a9e-2026-10-10.json) SHA `134aa9b45fa13b99966b6861c63b572cf7ae0d02c5dccaec1b9ca5aa6f8ef1a0`；[amd64重导入](evidence/certificate-hints-hosted-native-amd64-transfer-raw-c1581a9e-2026-10-10.json) SHA `134faf7cb93bdb49d0c12814e1c4785aa2ba48409e37e9a37a92b45fd63128f7`；[arm64构建](evidence/certificate-hints-hosted-native-arm64-raw-c1581a9e-2026-10-10.json) SHA `08b682fc3b365a000606d50d294434db9976a46bfaf6f061de63db8fb19f7675`；[arm64重导入](evidence/certificate-hints-hosted-native-arm64-transfer-raw-c1581a9e-2026-10-10.json) SHA `baa8d27336e6f9472ff6999b83c4a55992517b1aae0b87c37e65b781cacd1424`。业务最高963,334,144B（amd64构建）/ PID192（arm64重导入），每次2CPU / 2GiB / 256PID / 零swap、hard/OOM/PID触限0；普通阶段最高884,228,096B / PID197，允许swap但实际0。最高内存和PID来自不同报告，不拼成一次观测。

自己的准确镜像身份：amd64 `sha256:12c18b0562f940d5eb8934c54781b280bb4dc508598d597d7500f4d68e1ce67b` / 归档SHA `410ff876583dcd46582b4bcbe8611cf8494fe02296d4181358880ea1e5a4dcc6`；arm64 `sha256:e6ad3489f5af663aa4abd227c0ea69d961c5b8bdeff418142b80b48c54e65bcd` / 归档SHA `b411dac8569074951c67f7716fc0828cb3a95deb46183256336a18e1de1ebff0`。本机未下载/重算这些大归档，身份来自实际producer/重导入/publisher证据。

以成功Native任务为输入，已各启动 [amd6430分钟长测](https://github.com/warpdotsys/reader-dev/actions/runs/38065471515)、[arm6430分钟长测](https://github.com/warpdotsys/reader-dev/actions/runs/38065492523) 及 [准确原件/历史WebView/新镜像生成metadata三方](https://github.com/warpdotsys/reader-dev/actions/runs/38065495289)。脚本checkout仍为c158，镜像快照仍为0c5，输入Native任务仍为38063801166；不是稍后的UI通知候选。长测明确2CPU / 2GiB / 256PID / 零swap并仅给自己的临时parent cgroup设置high1.5GiB，包含冷请求、4生成用户GET/POST并发、超时/watchdog/自有worker崩溃恢复及每轮静默回收观察；三方先从公开归档提取原件并校验b26 SHA，不上传本地私有JAR或用户正文。**启动不等于通过**，当前还需各自终态和原始报告；原件TLS、真实站点认证及最新整镜像本机等仍未完成。

## 已知问题与回退

原件HTTPS/完整三方及编码扩展、真实站点认证/授权真实书完整旅程、WS/WSS、并发长测/冷启动/崩溃恢复、最新整镜像本机运行仍未齐；原件非ASCII/getBookGroups编码差异、OPDS后端未实现继续保留。证书诊断仅覆盖两个已识别码，不宣称涵盖过期、吊销、中间人或所有网络失败。

回退采用候选提交的独立revert：撤回内部hint字段、固定消息和新flag，保留原始报告及旧8项门槛；不要reset用户工作树，不删除数据，不恢复跨源认证泄漏，不忽略证书。当前原JAR/只读备份及58份受保护报告已在提交前重新核对，字节均未变，备份仍只读。
