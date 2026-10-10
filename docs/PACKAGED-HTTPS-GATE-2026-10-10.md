# 打包浏览器 HTTPS 发版门槛

日期：2026-10-10。**22f949cc自己的四条托管CI及双原生重导入已独立接受，12例打包HTTPS门槛真实完成；新fcd JAR另取得本机8场景实际Java API／上游代理HTTPS子集。未发版或部署。**

## 最新终态：22f949cc

[自己的托管产物、实际Java业务证据、已知风险与回退](READER-HTTPS-BUSINESS-2026-10-10.md)。525LinuxPython零跳过、198JVM／35原环境跳过、25实际生成UI零跳过，Full／Native新增TLS原始报告逐项独立核查，不借ec20绿灯。本机新fcd JAR实跑8场景、12目标请求／14CONNECT／178头字段，两用户Cookie／书源隔离、两证书负例均通过，52.39秒、2CPU2GiB零swap、峰值1,611,509,760B／PID240、soft high2468／hard触限0、自有清理0。

本机复用旧缓存运行时加新JAR，不是新整个镜像；该8场景业务链也不是托管工作流已有的worker门槛。新fcd的原件三方、真实账号与长测未完成。以下失败及候选章节保持各次捕获时的真实状态，不因新通过而抹除。

## 最新实际失败与修订候选

**e7236633自己的实际运行仍失败**：源码 `e7236633474868c6aaa939cfb71e872d483b6a6d`／测试合并 `5881352303f553776fc9867a3bbafcc22cc3c093`，API对比确认文件树一致。Java与Vue终态成功，但[Native](https://github.com/warpdotsys/reader-dev/actions/runs/38046230748)的amd64／arm64与[Full](https://github.com/warpdotsys/reader-dev/actions/runs/38046230859)均外部300秒超时／退出124。三个实际作业都只观察到首个same-get为TimeoutError、目标请求数0，未产生最终完整JSON；后续各例及总请求数未知，不把不完整观察改写成36例全失败或全程无请求。转移重导入与publisher未执行。保留[准确失败回执](evidence/packaged-tls-e7236633-failure-2026-10-10.json)，SHA `a6572723cd59c09c681d983510ae24a7e541cd1767173bd5d91c991c5b54441d`。异常由OSError变为TimeoutError不等于字体修订已完成验收。

本机生成单例j／k以同一冻结worker、只读home及浏览器、2CPU／2GiB／256PID／零swap重现启动超时；分别75.325／75.242秒、峰值1,439,424,512／1,397,673,984B、PID77／76。两次都未发生OOM或触限，精确自有运行时清理后清单0，未运行Reader JAR。j日志有dconf只读警告，但k只增加1MiB私有dconf临时区后仍超时，明确排除它作为单一修复。l的更广私有缓存尝试在浏览器启动前因生成挂载父目录权限失败，不属于HTTPS或浏览器结论。继续定位只读缓存／启动条件，不扩大总资源、不忽略证书或解除rootfs只读。

**本机最小目录对照已取得，托管新候选另验**：j/k/m/o仍超时；dconf、整体缓存、`.config`各自均不足。n/p的私有home可启动；仅查看生成目录的元数据后发现实际新建的是不带点的`/home/reader/camoufox`，而既有`.camoufox`挂载并非同一路径。q只增加这一个1MiB目录即可通过；r回到与j相同的只读home／缓存及既有私有字体配置条件，只增加该目录，11.118秒完成same-get，两实际HTTPS请求、两CONNECT、原生脚本生成标记和同源认证字段存在，峰值1,611,218,944B／PID167。2CPU／2GiB／256PID／零swap未变，hard max／OOM／PID触限0，自有容器清单0；soft `memory.high`事件751真实保留，不冒称全部压力事件为0。[九轮原始结果核查](evidence/readonly-browser-appdir-2026-10-10.json)，SHA `ac05f54a73693bd04013783d6dbb74ff92d830039dc929b05bdf02617bbcc0c5`。

上述是旧缓存运行时中冻结c3ef2486的源码单例，不是新JAR／新托管镜像、12例证书控制或Java API接受。原单例记录没有逐隧道源端口字段，仅内部报告目标端口集合属于代理；回执明确不宣称独立接受了逐隧道关联，也不补造字段。正式打包门槛继续保留更严格的完整逐端口字段核查。

新候选仅在现有TLS测试容器增加`/home/reader/camoufox:size=1m,mode=700,uid=10001,gid=10001`。外部容器属性和内部mountinfo／UID／GID／模式均必须核验，报告及publisher拒绝缺失的app-data隔离字段。没有开放整个home或增加`.config`／整体缓存，没有修改不可变浏览器字节、生产服务、主机信任或权限。28项生成报告守卫、77项结构守卫和完整525项Python／27.636秒／1项原Windows平台跳过通过；Bash语法与Python3.10源码解析通过。新提交自己的托管amd64／arm64及Full仍待实测，不能借r单例或ec20绿灯。

892205bd／tested a78a6d 的 Java 与 Vue 已终态成功，但 [Native](https://github.com/warpdotsys/reader-dev/actions/runs/38044601669) 的两个原生作业与 [Full](https://github.com/warpdotsys/reader-dev/actions/runs/38044601623) 的镜像作业均在新增门槛失败。三个作业／36次调用均为 OSError，实际目标请求／CONNECT隧道为0，不能作为任何HTTPS／证书控制接受。Full的UI报告缺失是TLS前置失败后的收集失败，不是实际UI测试执行后的失败。完整保留[生成失败回执](evidence/packaged-tls-892205bd-failure-2026-10-10.json)，SHA `8a46f7ec8c46f1c308c9a29b79a4a7759228124a03bce5196728bbfd6969db5d`；它来自实际日志内JSON行，不冒称重算了下载ZIP或独立接受清理。

**从缓存的锁定运行时依赖源码推断、尚待实测确认**：Camoufox `utils._generate_fontconfig` 首次启动会在 `INSTALL_DIR/fontconfig` 创建并写入小配置。缓存目录在本候选的只读根目录里且此前未单独挂临时区；此前本机i轮的整体私有缓存临时挂载允许这一写入，因此不能直接借用i轮通过。892版本只记录了异常类别，未记录errno／文件名，不能宣称已确认完整错误原因。

后续候选仅新增 `/home/reader/.cache/camoufox/fontconfig` 的1MiB、UID10001私有tmpfs，宿主信任、只读根目录、只读浏览器包、无外网、2CPU2GiB／256PID／零swap与完整24请求字段／两个证书控制均不放宽。控制器与内部mountinfo均须核验它，报告守卫拒绝缺失挂载；生成错误仅追加errno与有限路径类别，不记录任意路径／URL。27项报告守卫与76项结构守卫通过；完整524Python／32.375秒／失败0／1项原Windows POSIX跳过，脚本Bash语法通过。新的真实镜像运行仍待自己的CI，不把推断称为已修复。下面的523项／26项／75项数字为第一次门槛提交前的历史捕获。

## 已成功重建：ec20fb74

源码 `ec20fb7448c11a32e4a3e5d0cdceb8651c46b877`，实际合并测试快照 `7aada65be975e047b16618928dc913199b98f3c5`。GitHub 对比为前进一提交、文件差异为零；四条任务的源码身份、实际作业、托管 runner、XML 方法与跳过数及小产物均重新核验，不借 ac0bb598 的绿灯。

| 实际任务 | 独立接受的有限结果 | 回执 |
| --- | --- | --- |
| [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/38042474552) | Linux 497 Python／零跳过；194 JVM／35 个原环境跳过；两次干净 Boot JAR 字节一致 | [Java](evidence/origin-header-hosted-java-ec20fb74-2026-10-10.json) |
| [Vue 3](https://github.com/warpdotsys/reader-dev/actions/runs/38042474537) | 25 实际浏览器流程／零跳过，11 张生成截图；不等于本轮目视全部页面或生产登录 | [UI 与 Java](evidence/origin-header-hosted-ui-java-ec20fb74-2026-10-10.json) |
| [完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/38042474559) | 24 Cam／23 Chromium／3 helper、生成 Reader API 与默认界面；峰值 848,883,712B／PID193 | [Full](evidence/origin-header-hosted-full-ec20fb74-2026-10-10.json) |
| [双原生架构与重导入](https://github.com/warpdotsys/reader-dev/actions/runs/38042474595) | 六作业成功，amd64／arm64 与转移重导入、24 个旧格式 publisher 小 JSON；最大 872,742,912B／PID204 | [Native](evidence/origin-header-hosted-native-ec20fb74-2026-10-10.json) |

新 JAR：`a6e495aa3d3ae92b924429d0954c2d0e62c8a66fe82e379118a35dc51ed89d75`，285,671,158B／1,570 条目。此处是当前运行的重建报告与不同阶段身份交叉接受，不宣称已将此 JAR 下载到本机重新计算字节哈希。普通镜像任务允许 swap、实际使用 0，不能冒称本机零 swap／memory.high1.5GiB 的验收。

主 agent 本轮另外实际查看了这次 7aada 快照的390×844完整登录页和编辑后听书设置生成截图：这两张中文可读、登录控件可见。仅这两张的目视证据，不是全页面编码、真实账号或线上已部署版本证明；截图中的账号／听书源为生成数据。

小回执 SHA-256 依次为 `7dcba80b955aeac0e8561a1eafd2523143f2a0736c8eeb03d5144b4d42c9d458`、`47a1895c5546fd7c4eb9f6f58e828404e03c5809b3ded60a0f83dc5e4489c9f1`、`88a0ae41638e0681ae5201ed9fcbdf5988bc929e58afa430c1690e0ea25043b1`、`a9d8376b2abe1432f68184eff69e2edb013f1b36b32a1ca89d7862e9fb516567`。

## 新增候选：从实际产物读取，不用测试替身冒充浏览器

- `scripts/smoke-camoufox-tls.py` 校验待发版 JAR 的完整流式 SHA，唯一的 `BOOT-INF/classes/camoufox/worker.py` 及其与可读生产源码的字节 SHA，然后加载这些实际打包字节。
- 官方浏览器路径和基础策略由准确镜像只读查询；不假定两个架构的安装路径一致。仅在该次离线测试容器的 1MiB 私有 distribution 临时挂载中安装生成 CA。主机／镜像原策略不写入，不用 `ignoreHTTPSerrors`，不覆盖 worker 启动方法。临时挂载遮蔽原文件、不持久化；其内存仍计入容器限额，参见 [Docker 官方说明](https://docs.docker.com/engine/storage/tmpfs/)。
- 网络只有 `lo`；UID/GID10001、无能力、no-new-privileges、只读根目录；2 CPU／2GiB／256PID、swap 限额及使用均须为0。Reader 正式 smoke 与这项测试顺序执行，不在同一个 runner 上同时启动两份预算。
- 10 个正例、24 个目标请求：同／跨站303、307/308原始39字节UTF-8正文与声明的application/json媒体类型、原生页面脚本跳转、脚本资源跳转、Secure/HttpOnly/hostOnly/Path/SameSite Cookie。这里核查原始字节与媒体类型，不宣称该表单形状正文是合法JSON对象。逐请求保留字段、正文 SHA 和实际 TCP 源端口，匹配对应 CONNECT 的目标与双向传输，而不只接受“passed=true”。
- 两个负例必须明确得到 `SSL_ERROR_BAD_CERT_DOMAIN`／`SEC_ERROR_UNKNOWN_ISSUER`，并证明实际完成 TLS 握手尝试、没有发送 HTTP 请求。
- `scripts/verify-camoufox-tls.py` 在测试结束和 publisher 消费时分别重验原始观察。`BROWSER_TLS.json` 随 built／transferred 两架构产物传递；publisher 的 JSON 数从旧24增加为28，不用旧回执填充缺失报告。

Full workflow 同样要求该门槛，产物名 `bundled-browser-tls-generated`；它在 Reader 进程启动前完成。失败必须阻止后续打包接受，不会自动降低资源限制、忽略证书错误或退回旧全局规则头。

## 本机已验证与尚未验证

新增26项报告／流式哈希守卫都是明确的生成替身测试，不是实际浏览器证据；能拒绝错误 JAR/worker/源码身份、旧 Host、CONNECT 不匹配、认证头外溢、正文／媒体类型变化、Cookie跨站、脚本未执行、证书失败原因缺失、root／外网／swap／OOM／缺例及超限字段。完整523项 Python／失败0／1项原 Windows POSIX 跳过通过；75项流水线结构／删改失败控制通过；三个脚本的 Bash 语法检查通过。兼容镜像内 Python3.10，不调用仅3.11提供的 `hashlib.file_digest`。

此前真实源级 HTTPS 的[独立回执](evidence/origin-header-worker-https-2026-10-10.json)仅覆盖相同 c3ef2486 worker 的直接运行；本次新门槛不能借用它作为自己的打包运行结果。ec20 的镜像 CI 在门槛加入前执行，也不能借用作本门槛通过的证据。

仍待验证：本门槛自己在托管 amd64／arm64 的真实执行；新 a6e495 JAR 与准确原件的实际三方；Java CONNECT 出口／Reader业务的 HTTPS 全链路；WebSocket头作用域；最新整镜像本机运行、真实站点登录、长时资源、OPDS及生产流程。生成登录成功不代表用户线上账号成功。生产／发布授权不会被解释为这些未验证项已经完成。

## 保护与回退

本轮仅生成测试数据，不读取真实正文／凭据，不访问生产。原 JAR 和只读副本仍为 b26fb476；58个用户未提交报告的字节 SHA 均未变。官方 `uidmap` 已安装，当前版本 `1:4.17.4-2ubuntu3`；未调整已有映射范围或系统代理。

候选门槛回退仅撤回新增 TLS 脚本、报告消费与 workflow 门槛及其守卫，保留观察记录，不改变原 JAR／用户数据／已提交的 c3ef2486 生产头修正。不得为了恢复绿灯全局忽略 TLS 错误、重加跨源 Authorization 或删除其他容器。待其真实运行失败时保留诊断、修正测试或实现并重新取得自己提交的证据。
