# 原始 JAR 与恢复构建的隔离 WebView Cookie 差分

日期：2026-09-29。证据：[原始 JSON 报告](evidence/webview-cookie-netns-2026-09-29.json)。这是同一次、同一合成 `/render.html` 夹具驱动的**实际双 JAR 运行**，补充而不覆盖[此前的恢复版单独测试和历史差分](REMOTE-WEBVIEW-FALLBACK-DIFF-2026-09-29.md)。

## 样本与安全边界

- 原件 `reference/original/reader-pro-3.2.14.original.jar` 只读使用，SHA-256 为 `B26FB4769D689D98FF26408CE79A275D719F360906C84ACF52FF404E98030C8C`；恢复构建 `build/libs/reader-4.0.7.jar` 的受测 SHA-256 为 `0B1EBE9E92B53E7256E96069F76668957754336DD34AB1728A9B25AD8458A2CF`。后者在本机用 JDK 11、Gradle 6.1.1 离线 `bootJar` 成功构建，未对原 JAR 或三个旧工程做写入。
- 原 JAR 的 Vert.x 监听不能依赖 `reader.server.bindAddress` 限制。运行前由 `unshare --net --fork` 创建与 PID 1 不同的 Linux 网络命名空间，命名空间只有 `lo`，置为 UP 后只有 `127.0.0.1/8` 与 `::1/128`，没有外部接口；两个 Reader 进程及夹具都在该命名空间内。`scripts/run-webview-cookie-in-linux-netns.py` 在核对网络命名空间、接口、原 JAR 摘要后，降权到 UID/GID 65534 再执行差分脚本。普通 WSL 网络下直接调用该包装器已被拒绝，且没有生成报告。
- 两个 JAR 分别使用随机一次性账号、独立临时工作目录和本地夹具；进程结束后已核查没有残留 JAR 进程。没有加载生产 `storage/data`、账号或需登录书源；夹具不执行 JavaScript。报告仅保留合成数据、摘要和状态，未收录一次性凭据。
- WSL Ubuntu 当时没有系统 Java。为避免安装系统包，在 `/var/tmp/reader-jar-isolation.wNhi4V/jre` 解包 Ubuntu OpenJDK 11 包，并仅在该临时目录把指向 `/etc/java-11-openjdk` 的已存在配置链接改为包内相对链接；工具重复运行得到 0 个新增改动。JRE 版本为 `11.0.32.1+1-1ubuntu1~26.04`，与 Windows 构建所用 11.0.8 不同。这是运行环境差异，不应从本样本推断任意 JDK 版本的逐字节兼容。

## 同条件结果

| 观测 | 原始 JAR | 恢复构建 |
| --- | --- | --- |
| 4 次合成搜索 | 每次 HTTP 200、`isSuccess=true`、`errorMsg=""`、1 本书 | 相同 |
| `/render.html` 的 `js_source` | `null → null → null → document.title` | 相同 |
| `/render.html` 的 `Cookie` 请求头 | 空 → 空 → 空 → 空 | 空 → `session=alpha==` → 空 → 空 |

夹具第 1 次响应设置 `session=alpha==`，第 2 次响应删除它。恢复版第 2 次请求回放该 Cookie、第 3 次起不再回放；这是结构化 Cookie 修复的**有意行为差异**，不是与原 JAR 逐字节一致。两侧都把第 4 次书源的 `webJs` 字段传为 `js_source`，但夹具没有执行该脚本，因此不能声称实际脚本渲染兼容。

## 复现与剩余限制

先构建恢复版，再在已具备 `unshare`、`ip`、可用 JDK 11 与隔离网络权限的 Linux 主机上运行。下面的路径是本次 WSL 样本，不是通用安装路径；包装器必须在新的仅回环网络命名空间中运行，报告必须是 `/var/tmp` 下未使用的新文件名。**不要直接在普通主机运行原 JAR 差分脚本**：原 JAR 可能监听所有接口，单纯设置 `bindAddress` 或环境变量不形成隔离。

```powershell
.\gradlew.bat -PreaderWebUi=vue3 bootJar --offline --no-daemon --max-workers=1
wsl.exe -d Ubuntu -u root --exec unshare --net --fork python3 /mnt/c/Users/chong/Documents/Codex/2026-09-16/g-i-t/work/reader-pro-restored/scripts/run-webview-cookie-in-linux-netns.py --java /var/tmp/reader-jar-isolation.wNhi4V/jre/usr/lib/jvm/java-11-openjdk-amd64/bin/java --original /mnt/c/Users/chong/Documents/Codex/2026-09-16/g-i-t/work/reader-pro-restored/reference/original/reader-pro-3.2.14.original.jar --restored /mnt/c/Users/chong/Documents/Codex/2026-09-16/g-i-t/work/reader-pro-restored/build/libs/reader-4.0.7.jar --report /var/tmp/reader-webview-cookie-netns-NEW.json
```

**尚未验证**：原生产远程 WebView 不可用；固定旧参考镜像不能视为同版生产实例。本测试既未把旧远程服务和内置 Camoufox 加入同条件三方差分，也未验证真实书源、实际 JavaScript 执行、代理、登录态、超时、资源上限或生产数据兼容。它不构成候选发布或切换生产的许可。
