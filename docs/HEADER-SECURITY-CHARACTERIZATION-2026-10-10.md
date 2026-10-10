# 认证头、重定向与子资源：实际三方诊断

日期：2026-10-10。**已知安全问题，尚未修复；不是发布或认证安全接受报告。**

## 实际结果

准确原件 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c` 与自己的已构建恢复JAR `a873136863b8f9fba61dd4245325771e983b34e613f97b398f040b41fbb3f3cd`，分别连接锁定历史WebKit，再以同一恢复JAR运行内置Camoufox。源码提交为 `bb6afa4e2295a1bfabc88ae45ef8e68b364f631d`；这次只增加可读诊断工具和生成守卫，没有修改生产worker/JAR或做字节码补丁。

两个目标仅为私有网络命名空间的 `127.0.0.1` 和 `127.0.0.2` 随机端口，没有外部接口。Authorization固定为 `Bearer GENERATED_NOT_A_REAL_CREDENTIAL`，不是用户凭据；不使用起点账号、浏览器会话或真实正文。

| 生成用例 | 原件＋历史WebKit | 恢复＋历史WebKit | 恢复＋Camoufox |
| --- | --- | --- | --- |
| 同源GET→303→GET | 2请求，同源Authorization保留 | 同左 | 同左 |
| 跨源GET→303→GET | 2请求，**跨源仍带Authorization** | 同左 | 同左 |
| 跨源POST→303→GET | 仅初始POST，不跟随Location | 同左 | 2请求，**跨源GET仍带Authorization** |
| 同源页面＋同源/跨源两个脚本 | 3请求，两脚本实际执行，**跨源脚本带Authorization** | 同左 | 同左 |

追加12份实际Reader搜索结果全部HTTP200／`isSuccess=true`，四用例的完整ReturnData在三侧逐字JSON类型比较相同，原件和历史恢复的8份请求字段对及请求顺序也完全相同。**这些成功JSON没有揭示认证头外溢，不能据此判安全通过。** 原POST不跟随跳转是实际历史行为，不修改参考引擎使其追上Camoufox。

新增25份认证场景目标请求，保留全部309个解析字段对（字段原大小写、顺序、重复值），加15份原有五用例目标请求共40份；不是HTTP原始线字节。每请求最多64字段／8192 UTF-8名称和值字节、正文64KiB，每用例32请求／整个追加128请求，超限拒绝、不截断或转成功；正文只保存字节数和SHA。生成脚本要求同源和跨源两个脚本均实际执行，缺失资源不能被解释为“凭据没有外泄”。

## 已知问题与证据等级

- **已从JAR验证（P1）**：生成书源Authorization在跨源GET跳转和跨源脚本传出；恢复历史链路保持这个旧风险。Camoufox还会跟随历史POST没有跟随的303，并继续传出Authorization。没有证明真实用户的凭据已泄漏，不能把生成结果升级为真实账号事故。
- **已从可读源码核查**：worker把书源头设置在整个context上；普通POST还复制到导航覆盖头。历史参考引擎也使用context级extraHTTPHeaders。官方[Route覆盖文档](https://playwright.dev/python/docs/api/class-route#route-continue)说明覆盖头会随重定向传播；仅在初始导航加头或修改HTTP代理，不能先验保证HTTPS跳转安全。
- **已成功重建，但本轮没有改业务**：a873来自bb6自己的GitHub托管构建、完整镜像和双架构原生验收；详情见[默认POST修正自己的结果](TARGET-HEADER-DIFFERENTIAL-2026-10-10.md#修正自己的产物和第二轮三方已取得)。本轮只在缓存锁定runtime内只读挂载这份JAR，不能冒充最新完整镜像本机验收。
- **尚未验证/修复**：跨源认证头最小权限实现、HTTPS/TLS跳转、真实账号和Cookie认证、各种自定义token头、长期运行、最新整镜像本机及生产。当前PR56仍draft；本轮不合并、发版或部署，不能接受此认证路径供生产使用。不能以复制历史风险满足“兼容”；安全修正需另外保留同源业务、Cookie、方法和正文证据。

## 隔离与资源实测

自有l轮56.628秒；父级aggregate2CPU／2GiB、memory.high1.5GiB、256PID、零swap。峰值1,611,649,024B／PID246，high5108，max/OOM/PID触限0。PID余量仅10，短测不证明长期余量或无泄漏；不提高限额、重置峰值或把页缓存当应用RSS。

launcher UID1000；两OCI实际UID/GID10001、无附加组/能力、noNewPrivileges，只有lo，user/net共享而mount/PID/IPC/UTS分开。原件、恢复JAR、脚本、依赖和参考浏览器只读，新storage/tmpfs。主机独立确认历史参考停止/PID0后才准许Camoufox；结束后两个自有容器独立stopped/PID0并删除，清单0。PID1只收养回收36个已死孤儿，排除直接Node子进程；不杀日常Reader、改代理/映射或重启WSL。

uidmap `1:4.17.4-2ubuntu3` 已到位；newuidmap/newgidmap可用。原有subuid/subgid SHA仍分别为 `d796e52bc335df4e55114fad949f19850e6b4008cf07bf8d53a4e88936be9cbd`，未扩映射。

## 可重复诊断、原始结果与回退

可读入口 `scripts/compare-webview-cookie.py --characterize-header-security` 必须与准确原件隔离、真实历史renderer、Camoufox、脚本/POST、全头观测及主机hand-off同时使用，不能在普通宿主模式启动原件。与UTF-8/metadata专项分开，不放宽其旧严格门槛；旧五用例必须仍通过。完整诊断后**固定退出1**，strict/security/parity/真实认证接受仍false；宿主“诊断已完成”不能转换成业务或安全绿灯。

例（仅用于已经独立核验的私有lo命名空间，原件SHA守卫和renderer停止交接仍强制）：

```sh
python3 scripts/compare-webview-cookie.py \
  --java /opt/java/openjdk/bin/java --original /verification-inputs/original.jar \
  --restored /verification-inputs/restored.jar --report /storage/new-header-security.json \
  --original-network-isolated --exercise-script --exercise-post \
  --archived-renderer-base http://127.0.0.1:8050 --camoufox-python /usr/bin/python3 \
  --phase-handoff-dir /storage/new-header-security-phases \
  --observe-target-headers --characterize-header-security
```

本轮锁定历史manifest `sha256:88c250043bd33715ca372b749c1dca055c14e4550882caf968bd7af933d53573`、缓存runtime image `sha256:401d895c31a39634bf29654a7fbb29527272d287c8c1219b12794bbe1970bc44`；不说明原JAR生产使用的参考引擎恰为此版本。原始l报告SHA `c2d54cae78f8e733cd7b7c69694dc9400909c02f94bf4e9d331ff4eecd6ec56e`。执行comparator SHA `de02ec3c468b73e46b574ea65ca0d6aa380a0f3ebf5b42b0f5e0d115a1914d56`，新fixture SHA `6571761a214f68c9cde1f5ccd7a6a5de96582cd24e6659c0b2638b3d6ba4d44f`。

[独立有限回执，含25请求字段对与12份实际生成ReturnData](evidence/header-security-characterization-2026-10-10.json)，SHA `3831e361daf899d05e8fac89b55bcf1c9bd59f688699664d28840b3df5164eb0`／57,912B。独立复验源码SHA `77e4290486265588e796ac73d41aefe364c0c4dd2e7679ab9e5656336fa63b06`，重新判原件/历史请求、生成值外溢、完整JSON、既有五用例及隔离预算，不仅信诊断boolean。原件/只读备份/保留CI字节SHA和58个用户dirty报告均未变。

新增10项纯生成守卫，本机Python481项／24.432秒通过、原Windows POSIX跳过1；守卫覆盖缺隔离不得bind/start、定义固定凭据、脚本必须执行、缺/重复/越界观测不得转通过、错误API/脚本JSON、快照与有限字段及CLIfail-fast。它们不是实际浏览器证据，自己的托管结果须另验。本轮不改worker，回退只撤销诊断入口/新fixture及测试，不覆盖原件、用户报告或生产数据；回退不会修复上述认证风险。

完成后外部再次确认l unit inactive/MainPID0、runc清单空，按真实路径/所有权/单链接/只读模式/完整SHA，仅删除l临时输入目录中两份JAR副本358,579,951B（约342MiB）。保留原件、只读备份及b/c托管下载，删除前后完整SHA不变，副本可再生成；不删除其他缓存目录。清理回执SHA `92a0fa9d71674909801beba863bc413abea34bd7bd02ddd50d321a8f179dca05`。
