# 浏览器网络拒绝的脱敏诊断

## 已从当前源码与生成样本复现

真实起点 Reader 首两轮只得到聚合的网络拒绝。用户确认系统返回的 `198.18.1.220` 是代理 Fake-IP，不是起点真实内网地址；此诊断不否定该说明，也不把保留网段直接当成可安全连接的公网地址。

本轮先在未修复源码上运行生成回归，4 项分类断言失败；另用 JDK source-file 模式的微型生成 worker，经真实出口代理请求一个被禁止的 loopback CONNECT，收到 403 后返回导航错误。父级没有保留类型化拒绝，新增断言也失败。两套原始 XML 保留在 `build/browser-network-diagnostic-20261004-a/baseline/`。这是父进程协议回归，不是运行了真实 Camoufox 浏览器，更不是原 JAR 行为一致性证据。

## 源码修复与安全边界

- 网络策略保留中文解释，并提供固定原因代码：非法 URL/协议、带账号 URL、无效主机/端口、内网主机、DNS 失败/空结果、保留 benchmark 网段、其他非公网地址。
- 错误仅附上规范化主机的 SHA-256 前 16 个十六进制字符，不附原始主机、解析 IP、URL 路径/参数/fragment、请求头、Cookie 或底层 DNS 异常文本。该指纹用于已知公开主机的离线比对，不是加密或不可猜测的匿名标识。
- 两个 renderer 保留首个策略拒绝；Camoufox worker 的导航超时/错误不能掩盖它。代理仍返回泛化 403，不把诊断和凭据发给源站。
- 整个 DNS 答案集仍需校验，混合公网/私网仍失败。`198.18.0.0/15` 仅被区分为 `BENCHMARK_RANGE`，**没有放行**；不改系统 DNS/hosts、默认引擎、Docker 网络或私网开关。不能为解决 Fake-IP 而全局设置 `READER_BROWSER_ALLOW_PRIVATE_NETWORKS=true`。
- ReturnData 的 HTTP/`isSuccess`/`data` 结构不变，拒绝时的 `errorMsg` 有意增加脱敏分类。此处不是与原始 JAR 完全相同的错误文本。

## 已成功重建与定向回归

修复后 7 套件、20 项、0 失败/错误/跳过，覆盖上述生成样本以及既有 HTTP/SOCKS 固定 IP、代理凭据隔离、网络守卫、worker 输出上限。新 HTTP 和 CONNECT 拒绝用例确认 403、类型化分类与 16 位主机指纹，且响应和诊断中没有生成凭据或路径。生成 worker 回归实际通过 1.022 秒；真实 Camoufox 合约仍须托管运行，不拿它替代。

首轮增量编译因枚举漏项失败，修正后定向构建实际成功；保留失败记录，不将构建错误算成网站故障。发布链路 47 项本机测试及 actionlint 通过；actionlint 此处未执行 shellcheck/pyflakes。全量构建和公开页复测结果在后续验收段记录。

全量 `-PreaderWebUi=vue3 test bootJar` 成功，38 套件、129 项、0 失败/错误；12 项真实 Camoufox 与 9 项 Chromium 合约因对应本机环境未配置而跳过，不能计入成功执行。构建实际 4 分 51 秒、8 个任务执行，包含 Java/Kotlin 重编译。独立副本 `build/browser-network-diagnostic-20261004-a/candidate-network-diagnostic.jar` 的 SHA-256 为 `A94A68ACA5EFFD744EFFE8E51DD0E0101FF4C2568A862881E9714260B1401CDF`。此散列是本机未发布候选，不是托管制品身份。

## 已实跑的匿名公开页诊断

新隔离存储、生成 Reader 账号、`127.0.0.1:18944`、768 MiB JVM/2 处理器，只原样导入官方桌面书源；没有导入任何真实会话字段、读取章节正文或请求付费入口。使用此前九个公开资源主机的进程级 DNS 快照，不动系统设置；它有时效/覆盖范围限制，不是生产 DNS 修复方案。

实际单源搜索 9.166 秒、HTTP 200、`isSuccess=false`、精确书名 0 项。新诊断保留 `DNS_FAILURE` 与主机指纹 `191347bfe55d0ca9`，它不匹配快照里的九个已知公开主机。**这不是将 Fake-IP 认作实际内网地址**；也不能由该结果断言目标站公网 DNS 故障，未覆盖的动态主机同样会在此次专用快照环境解析失败。具体主机仍未定位，更不能把匿名一轮的原因套回先前认证两轮。

另一次不带 Cookie 的普通 HTTPS 请求同一公开搜索页返回 202，没有提取到资源主机；未保存 HTML，不推断挑战类型或认证状态。匿名探针只保留分类/指纹/状态，不保存原始错误、页面 HTML、正文或请求头。finally 清除后 Cookie 列表仍为 0，持有原生句柄请求结束自有 JVM PID 25168 的进程树，JVM 与端口已实际关闭；没有独立枚举每个退出的后代 PID，不冒充完整进程审计。用户 Reader 的 18931/PID 61244 监听保留。没有复用浏览器登录态。

[脱敏证据](evidence/browser-network-diagnostic-2026-10-04.json)记录失败与通过 XML 散列、真实执行/跳过数，以及匿名结果。新诊断可用于后续精准比对，但 Reader 起点搜索尚未修好。

可重复定向命令（使用本工程的 JDK 11、Wrapper 与隔离 Gradle 缓存）：

```text
gradlew.bat test --tests com.htmake.reader.utils.BrowserNetworkPolicyDiagnosticTest --tests com.htmake.reader.utils.CamoufoxWebviewRendererNetworkDiagnosticTest --tests com.htmake.reader.utils.BrowserNetworkPolicyTest --tests com.htmake.reader.utils.BrowserEgressProxyTest --tests com.htmake.reader.utils.BrowserUpstreamProxyTest --tests com.htmake.reader.utils.LocalWebviewRendererGuardTest --tests com.htmake.reader.utils.CamoufoxWebviewRendererOutputLimitTest --offline --no-daemon --max-workers=2
```

完整候选构建命令：`gradlew.bat -PreaderWebUi=vue3 test bootJar --offline --no-daemon --max-workers=2`；前置为已构建且验证过的 `web-vue3/dist`、本工程 JDK/缓存。离线缓存不存在时先按标准构建文档准备依赖，不能伪造缺失依赖或省略 UI。

## 尚未验证与回滚

尚未证明真实认证 Cookie 在内置引擎到达起点、Reader 搜索/正文解析通过、全部动态资源被正确解析，或真实认证三方一致。没有重新导入真实凭据、购买章节、修改网站账号、上传私有数据或部署候选。后续公开匿名诊断也不能当作认证验收。

此增量只改可读源码，无字节码补丁或存储迁移；可在干净工作树撤回网络诊断增量并重建上一验收快照，保留默认拒绝策略和远程回退。禁止对用户当前脏目录 reset/checkout。上一 `fdad2c0f` 的四条托管绿灯只覆盖 SPA 增量，不覆盖本轮诊断。
