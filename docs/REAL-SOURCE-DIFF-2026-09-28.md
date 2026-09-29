# 公开真实书源首轮差分（2026-09-28）

历史执行命令（**不要在当前 Windows 主机上照抄重跑**）：原 JAR 的 Vert.x 监听没有采用恢复版的回环绑定参数，而测试所用 JDK 11 可被有效入站规则放行到 Private/Public 网络的任意 TCP 端口；此前本报告的“两个进程隔离工作目录”不等于网络入站隔离。新原 JAR 差分须先建立独立网络命名空间或明确阻断入站；本轮只读核查未改防火墙，也没有重新运行原 JAR。

```powershell
$env:JAVA_HOME='C:\Users\chong\Documents\Codex\2026-09-16\g-i-t\work\reader-pro-restored\.tools\jdk-11.0.8'
$env:GRADLE_USER_HOME='C:\Users\chong\.gradle'
.\gradlew.bat -PreaderWebUi=vue3 bootJar --no-daemon --max-workers=1
python -B scripts/compare-real-public-explore.py
```

脚本使用原始 JAR 的只读备份和刚构建的恢复 JAR；两个 Reader 进程分别使用自动清理的临时工作目录、随机合成账号，不连接生产 Reader，也不读取 `storage/data`。唯一外部目标是公开的 `https://m.jjjxsw.com/txt/` 目录页，以仅包含 `bookList/name/author/bookUrl` 的测试书源进行 `/reader3/exploreBook` POST。测试不登录第三方站点、不下载章节、不保存站点正文或用户书源。输出只含状态、字段集合、数量与 SHA-256 摘要。

## 已从原始 JAR 验证

- 原始 `reader-pro-3.2.14.jar` SHA-256：`B26FB4769D689D98FF26408CE79A275D719F360906C84ACF52FF404E98030C8C`。
- 公开站点测试前后均为 HTTP 200，页面长 15,341 字节，原始页面 SHA-256 均为 `CAAEC4912D4DB51EA531E86CF1FDF4933D30C88350B240FAEA9125D2CBECEF8C`。这只证明两个边界取样一致，不能证明中间每次响应逐字节一致。
- 原 JAR 返回 HTTP 200、`isSuccess=true`、`errorMsg=""`；解析 10 本书，10 本均有 `bookUrl`。结果数组的规范化 JSON SHA-256 为 `8C0C0F23A2D8C935A3488290E534E5F426799631E0FE3F851EAF1E55F425576A`。

## 已成功重建

- 当前本机构建 JAR SHA-256：`83AD953938980A699F7CF486049213E2F48E5D6614AE52B71A71540D612D5313`；它标识本次受测产物，不是正式镜像摘要，也不证明字节级可重复构建。
- 恢复版同样返回 HTTP 200、`isSuccess=true`、`errorMsg=""`；解析 10 本书，10 本均有 `bookUrl`，规范化数据摘要与原 JAR 相同。
- 首项字段集合均为 `author, bookUrl, intro, latestChapterTitle, name, origin, originName, originOrder, time, tocUrl, type, wordCount`。本样本中未观察到数据兼容差异。

## 已从 GitHub 托管 runner 的单容器验证

手动开启 `browser-image.yml` 的 `real_public_source` 选项后，[作业 36367557675](https://github.com/warpdotsys/reader-dev/actions/runs/36367557675) 完成 JAR 构建、真实 Camoufox 合约、完整镜像构建及容器内合成书源冒烟，并在同一受限容器中通过镜像内的 Camoufox 请求上述真实公开目录。这个额外测试调用 `/reader3/searchBook`，测试书源的 `searchUrl` 明确设置 `{"webView": true}`；返回 HTTP 200、`isSuccess=true`、`errorMsg=""`、10 本书，书名与 URL 投影 SHA-256 为 `F49D8B6E0A207FC31B8447D99287CE9B23708E56F7059E85C8964C0B4F15B6E2`。容器使用 2 GiB 内存、3 GiB memory+swap、256 PIDs 和 2 CPU 上限。本测试由托管 runner 执行，不依赖用户电脑或生产主机的 Chrome，也没有独立运行的 WebView 服务。

实站步骤是显式手动选项；普通 CI 仍运行确定性的合成书源，以免外部站点临时停机被误判为源码回归。站点列表会随时间变化，不能把两次不同环境的结果摘要视为严格字节差分；本节证明的是容器内公开实站的正向功能，而不是原 JAR 对 Camoufox 的等价性。

## 尚未验证

- 原 JAR／恢复版的一致性样本是公开静态目录的真实 HTTP/规则解析，未启用 `webView`；另一个容器内样本启用了内置 Camoufox，但没有同条件运行原 JAR 和旧远程 WebView。两项证据不能拼接成三方等价，且站点本身不是依赖 JavaScript 或登录态的样本。
- 还需覆盖真实脚本、重定向、登录态 Cookie、跨用户隔离、超时、异常响应、来源限制与并发资源。在生产主机上，旧 Reader 配置引用的远程 WebView 服务当前不可用；恢复其可复现对照前，不应声称完成三方验证。
- 页面可能随站点更新而变化。脚本每次都记录前后上游摘要；如果摘要变动，即便两个 JAR 的结果不同也须先排除内容漂移。测试成功不授权持续高频抓取该站。
