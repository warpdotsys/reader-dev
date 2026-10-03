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

## 同日追加：POST 请求形态实测

[新增原始 JSON](evidence/webview-post-netns-2026-09-29.json)来自同一隔离方式的**新双 JAR 运行**，原件和恢复 JAR 的 SHA-256 与上面一致。包装器只有显式加 `--exercise-post` 才增加第 5 次搜索；因此上面的四次搜索报告与复现命令仍可独立保留。第 5 次书源 URL 选项为 `webView=true`、`method=POST`、`body=q=post`、`headers.X-Fixture=synthetic` 和 `webJs=document.title`。

| 观测 | 原始 JAR | 恢复构建 |
| --- | --- | --- |
| 第 5 次搜索 | HTTP 200、`isSuccess=true`、`errorMsg=""`、1 本固定书 | 相同 |
| 传给 `/render.html` 的请求字段 | `http_method=POST`、`body=q=post`、`headers.X-Fixture=synthetic`、`js_source=document.title` | 相同 |
| 第 1–5 次 Cookie 请求头 | 全为空 | 空 → `session=alpha==` → 空 → 空 → 空 |

新增夹具同时记录前四次的请求字段：两边均为 `http_method=GET`，`body` 和 `X-Fixture` 均为空。JSON 文件已经与 WSL 中未改写的原始报告做语义比对；只保存合成头和请求体，不保存账号、访问令牌或生产 Cookie。复现时使用新的 `/var/tmp` 报告名，并在上面 `wsl.exe ...` 命令结尾添加 `--exercise-post`。

**严格边界**：这是两个 Reader 向合成远程渲染器 `/render.html` 发送的请求**字段**差分。夹具不访问目标 `/search`，所以并未证明旧远程服务或内置 Camoufox 真正向目标站发送了 POST，也未证明脚本实际执行、响应正文等价或原生产实例同版。上述空 Cookie 与结构化 Cookie 回放仍是有意差异。

## 同日追加：实际历史 WebKit 服务的双 JAR 差分

本节于 2026-10-03 整理入库，运行发生在 2026-09-29。[独立 JSON 报告](evidence/archived-webkit-pair-2026-09-29.json)来自两个 JAR 实际调用固定摘要的历史 WebView 镜像，而非上面的模拟 `/render.html`。原始与恢复 JAR 摘要仍与本文开头一致。受测镜像为 `hectorqin/remote-webview@sha256:b61d8e86f69a743baa06aadb58cdba6d1e11c5baa45cec05a908d541ce9a684a`，平台为 `linux/amd64`；这只是可追溯的历史实现参考，未证明与原生产远程实例相同。

旧服务容器使用 `--network none --memory=2g --cpus=2 --pids-limit=256 --shm-size=1g --cap-drop ALL --security-opt no-new-privileges`。由宿主机 `nsenter` 进入该容器的网络命名空间后，包装器再次核对命名空间与 PID 1 不同、仅有 `lo`，校验原 JAR 摘要，然后降权到 UID/GID 65534。两个 Reader 和目标夹具均使用该私有回环网络，旧服务位于 `127.0.0.1:8050`；没有生产数据或外部路由。包装器把已验证的网络命名空间 inode 传给降权后的探针，并由探针核对自身 inode 与接口，解决了非 root 无权读取 `/proc/1/ns/net` 的问题。

| 目标端与 Reader 观测 | 原始 JAR | 恢复构建 |
| --- | --- | --- |
| 5 次搜索的结果 | 每次 HTTP 200、`isSuccess=true`、`errorMsg=""`、1 本固定书 | 相同 |
| 前 4 次目标请求 | GET，空请求体，无测试头 | 相同 |
| 第 5 次目标请求 | POST，`q=post`，`X-Fixture=synthetic` | 相同 |
| 5 次目标 Cookie 请求头 | 全为空 | 相同 |
| 第 4、5 次脚本结果 | 固定书名改写成功并被 Reader 解析 | 相同 |

第 4、5 次目标 HTML 的书名为 `WebView脚本原始书`；配置的 `webJs` 返回 `document.documentElement.outerHTML.replace('WebView脚本原始书','WebView差分书')`。探针只接受解析后的 `WebView差分书`，因此未执行脚本、直接解析目标 HTML 的路径会失败。这证明此合成脚本结果参与了实际旧引擎链路。目标夹具没有截获 Reader 到旧服务的 `js_source` 字段，报告以 `jsSourceDirectlyObserved=false` 明示；`renderScriptSources` 的空值也不应解释成 Reader 未发送脚本。

目标第 1 次响应设置 Cookie、第 2 次删除 Cookie，但本次两个 JAR 的目标 Cookie 均为空。与上面的模拟响应差分相比，观测位置已由 `/render.html` 改为实际目标 `/search`，且历史服务每次渲染使用新上下文、未将目标 `Set-Cookie` 转发到 Reader。不得把两个样本的 Cookie 序列合并为同一个行为结论。

### 复现实际旧引擎模式

在隔离测试主机上准备本地原件、恢复 JAR、可执行的 JDK 11 和 Docker。原件保持本地只读；报告使用新的 `/var/tmp` 文件名。以下命令的文件路径需要替换成测试主机实际路径，容器名也应使用本次专用名字。

```bash
docker run -d --name reader-archived-diff \
  --platform linux/amd64 --network none --memory=2g --cpus=2 \
  --pids-limit=256 --shm-size=1g --cap-drop ALL \
  --security-opt no-new-privileges --restart no \
  hectorqin/remote-webview@sha256:b61d8e86f69a743baa06aadb58cdba6d1e11c5baa45cec05a908d541ce9a684a
renderer_pid=$(docker inspect --format '{{.State.Pid}}' reader-archived-diff)
sudo nsenter --net=/proc/$renderer_pid/ns/net -- \
  python3 /absolute/reader-pro-restored/scripts/run-webview-cookie-in-linux-netns.py \
  --java /absolute/jdk-11/bin/java \
  --original /absolute/reader-pro-3.2.14.original.jar \
  --restored /absolute/reader-4.0.7.jar \
  --report /var/tmp/reader-archived-webkit-NEW.json \
  --exercise-post --archived-renderer
docker stop reader-archived-diff
docker rm reader-archived-diff
```

**尚未验证**：本节没有加入内置 Camoufox；未覆盖真实登录书源、完整脚本语义、代理、编码、超时、并发与长期资源曲线，也不证明原生产远程服务等价。测试容器和临时 Docker 缓存已清理，独立报告保留并与入库 JSON 做语义核对。新模式不改变默认合成模式；恢复版单独运行的合成模式也已回归，Cookie 为 `空 → session=alpha== → 空 → 空 → 空`，POST 字段保持一致。
