# 远程 WebView 回退的 Cookie 与脚本字段差分

日期：2026-09-29。测试只在本机回环端口上运行两个隔离的 Reader 进程和一个合成 `/render.html` 服务；使用随机一次性账号和临时工作目录，不连接生产数据、第三方书源或真实远程 WebView。模拟服务返回固定书目，并在第一笔响应设置 `session=alpha==`，第二笔响应删除它。第四笔搜索的书源附带 `webJs: document.title`。夹具记录请求形态，不执行浏览器脚本；因此本测试**只证明字段传递**，不是脚本执行兼容性验收。

命令：

```powershell
$env:READER_APP_WEBVIEWRENDERER='remote'
python -B scripts/compare-webview-cookie.py --exercise-script --report build/webview-script-diff-capability-20260929.json
```

| 样本 | 原始 JAR | 修复后恢复构建 |
| --- | --- | --- |
| SHA-256 | `B26FB4769D689D98FF26408CE79A275D719F360906C84ACF52FF404E98030C8C` | `2C2E24563100AD5CB9B3066E1F70B8A312A6EA7E14D2C30349992276714D5F26` |
| 4 次搜索 | 均 HTTP 200、`isSuccess=true`、`errorMsg=""`、1 本固定书目 | 相同 |
| `/render.html` 的 `js_source` | `null, null, null, document.title` | 相同 |
| `/render.html` 的 `Cookie` 请求头 | 空、空、空、空 | 空、`session=alpha==`、空、空 |

**已从原始 JAR 验证**：上述脚本字段和 Cookie 序列。**已成功重建**：恢复构建在远程回退模式保留脚本字段，同时在设置、回放、删除 Cookie 的序列中与既定的结构化 Cookie 修复一致。Cookie 差异是有意的，不称为逐字节兼容。

修复后本机 JDK 11、Gradle 6.1.1 离线 `test` 成功：32 个套件、86 项测试、0 失败、0 错误、14 跳过；定向 `ReaderAdapterWebviewTest` 两项均执行并通过。跳过项仍需以 Linux 托管 runner 的真实浏览器作业补验。

差分发现修复前的较早本机恢复 JAR（SHA-256 `A1F6E1BC8FD094232C65A3CA6E80207EA5595744DF8B40F68249074D4156AB96`）在强制远程回退时四次 Cookie 全为空：`AnalyzeUrl` 为内置浏览器跳过了旧式请求头，却也跳过了远程渲染器需要的 Cookie。现在由渲染器显式声明是否自行管理结构化 Cookie，适配器按该能力决定是否注入旧式请求头；未声明的新渲染器默认保留请求头。两份较早本机 JAR 分别保存在 `.tools/previous-builds/reader-4.0.7-A1F6E1BC.jar` 和 `.tools/previous-builds/reader-4.0.7-EEBD0374.jar`，均未覆盖或删除；它们不是原始 JAR。

**尚未验证**：原生产远程 WebView 不可用，本夹具不代表其版本或行为；固定 Docker Hub 旧参考镜像与原生产实例不能画等号。此次没有对三者做同条件真实书源、实际 JavaScript 执行、代理、超时或长时间并发差分。当前源码仍需 GitHub 托管 runner 全量测试与完整镜像烟测，不因本地脚本通过而发布。
