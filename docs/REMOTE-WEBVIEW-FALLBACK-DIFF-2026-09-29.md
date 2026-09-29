# 远程 WebView 回退的 Cookie 与脚本字段差分

日期：2026-09-29。两次 Reader 请求与合成 `/render.html` 夹具均使用 `127.0.0.1`，随机一次性账号和临时工作目录；不连接生产数据、第三方书源或真实远程 WebView。**安全边界修正**：原 JAR 的 `RestVerticle` 字节码调用 Vert.x 的 `listen(port, handler)`，不读取恢复版新加的 `reader.server.bindAddress`。因此此前“两个 Reader 进程只监听本机回环”的表述未经证实；当时未记录主机防火墙状态，也没有证据说明端口被外部访问。模拟服务返回固定书目，并在第一笔响应设置 `session=alpha==`，第二笔响应删除它。第四笔搜索的书源附带 `webJs: document.title`。夹具记录请求形态，不执行浏览器脚本；因此本测试**只证明字段传递**，不是脚本执行兼容性验收。

历史命令（只用于说明当时如何得到下表；**不要在未隔离的主机上照抄重跑**）：

```powershell
$env:READER_APP_WEBVIEWRENDERER='remote'
python -B scripts/compare-webview-cookie.py --exercise-script --report build/webview-script-diff-capability-20260929.json
```

差分脚本现要求显式选择安全模式，且拒绝覆盖已有报告。本机只运行恢复版时使用 `--restored-only`；只有独立核验网络命名空间或入站阻断后，才能设置 `READER_ORIGINAL_JAR_NETWORK_ISOLATED=confirmed` 并以 `--original-network-isolated` 运行原 JAR。标记和参数都不会创建隔离或防火墙规则。其他 25 条 Python 和 3 条 PowerShell 原 JAR 差分脚本也加入了相同启动前检查；PDF 夹具模式、仅恢复版诊断不受影响。2026-09-29 提权**只读**核查发现，测试所用 JDK 11 `java.exe` 在有效入站规则中获准于 Private/Public 配置文件接收任意 TCP 本地端口、任意远端地址的连接；防火墙总体默认阻断入站并不能覆盖这条程序放行规则。因此本轮没有重新启动原 JAR，也没有修改防火墙。既往测试是否发生外部访问没有可用证据，不应将可能暴露误写为已遭访问。

```powershell
$env:READER_APP_WEBVIEWRENDERER='remote'
python -B scripts/compare-webview-cookie.py --restored-only --exercise-script --report build/webview-cookie-candidate-bd786e45-20260929.json
```

**已成功重建的新样本**：当前 `bd786e45` 候选源码本机离线 `bootJar` 成功，受测恢复 JAR SHA-256 为 `6484F5CBAF701ED5DB1165F7B22B3EC4370F4E3BD8E7C6505492AC1AD2D3B781`。四次搜索均 HTTP 200、`isSuccess=true`、`errorMsg=""`，各得一本固定书；向夹具发送的 Cookie 序列为 `空 → session=alpha== → 空 → 空`，`js_source` 序列为 `null → null → null → document.title`。本次 `originalExecuted=false`，原 JAR 仅做只读 SHA-256 核对；下面的原 JAR 结果属于较早的历史观测，不得拼成此次同条件差分。

| 样本 | 原始 JAR | 修复后恢复构建 |
| --- | --- | --- |
| SHA-256 | `B26FB4769D689D98FF26408CE79A275D719F360906C84ACF52FF404E98030C8C` | `2C2E24563100AD5CB9B3066E1F70B8A312A6EA7E14D2C30349992276714D5F26` |
| 4 次搜索 | 均 HTTP 200、`isSuccess=true`、`errorMsg=""`、1 本固定书目 | 相同 |
| `/render.html` 的 `js_source` | `null, null, null, document.title` | 相同 |
| `/render.html` 的 `Cookie` 请求头 | 空、空、空、空 | 空、`session=alpha==`、空、空 |

**已从原始 JAR 验证**：上述脚本字段和 Cookie 序列。**已成功重建**：恢复构建在远程回退模式保留脚本字段，同时在设置、回放、删除 Cookie 的序列中与既定的结构化 Cookie 修复一致。Cookie 差异是有意的，不称为逐字节兼容。

修复后本机 JDK 11、Gradle 6.1.1 离线 `test` 成功：32 个套件、86 项测试、0 失败、0 错误、14 跳过；定向 `ReaderAdapterWebviewTest` 两项均执行并通过。跳过项仍需以 Linux 托管 runner 的真实浏览器作业补验。

差分发现修复前的较早本机恢复 JAR（SHA-256 `A1F6E1BC8FD094232C65A3CA6E80207EA5595744DF8B40F68249074D4156AB96`）在强制远程回退时四次 Cookie 全为空：`AnalyzeUrl` 为内置浏览器跳过了旧式请求头，却也跳过了远程渲染器需要的 Cookie。现在由渲染器显式声明是否自行管理结构化 Cookie，适配器按该能力决定是否注入旧式请求头；未声明的新渲染器默认保留请求头。两份较早本机 JAR 分别保存在 `.tools/previous-builds/reader-4.0.7-A1F6E1BC.jar` 和 `.tools/previous-builds/reader-4.0.7-EEBD0374.jar`，均未覆盖或删除；它们不是原始 JAR。

**尚未验证**：原生产远程 WebView 不可用，本夹具不代表其版本或行为；固定 Docker Hub 旧参考镜像与原生产实例不能画等号。此次没有对三者做同条件真实书源、实际 JavaScript 执行、代理、超时或长时间并发差分。包含 `81f14c19` 显式能力声明的 `207711f6` 已通过 [GitHub 托管 runner 全量测试与完整镜像烟测](https://github.com/warpdotsys/reader-dev/actions/runs/36513001239)；这仍不是原生产远程服务的同条件验证，也不构成发布许可。
