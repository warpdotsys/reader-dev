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

## 2026-09-29 同一次托管运行的双侧公开页观测

[作业 36529322389](https://github.com/warpdotsys/reader-dev/actions/runs/36529322389)在同一提交 `e128bcb8` 上并行运行固定旧远程 WebView **参考镜像**与完整 Reader/Camoufox 镜像，使用相同的无凭据公开目录 URL。参考镜像直接渲染 HTTP 200，另一次页面内 JavaScript 选择器读到 **10** 个书目 DOM 节点；Reader 在 2 GiB 内存、256 PID、2 CPU 限制的单容器中，以 `webView=true` 测试书源搜索返回 HTTP 200、`isSuccess=true`、`errorMsg=""` 和 **10** 本书，书名/URL 投影 SHA-256 为 `46c56e39a396824f6cc2d7ff920791f5f6810419c3d60d2e271740d7817ce561`。比较制品记录两次观测相隔 433.381 秒，`countEqual=true`，但明确标记 `fullResponseParityProven=false`、`originalJarCompared=false`、`referenceProvenProduction=false`。站点可变、请求阶段不同、旧参考镜像并非已证实的原生产实例；数量一致仅是有限的兼容线索，不是三方等价证明。作业只上传无正文、凭据及 Cookie 值的摘要。

`52d30125` 的[首次数组投影作业](https://github.com/warpdotsys/reader-dev/actions/runs/36531083960)在两个子作业中均得到 10 本及相同的书名/URL 投影摘要，但参考镜像额外的纯数字脚本偶发不可解析，导致整次比较作业失败。该失败是探针形式不稳定，不能写成产品解析失败或整轮验收通过。移除纯数字步骤后，`a53cc7be` 的[修正配对作业](https://github.com/warpdotsys/reader-dev/actions/runs/36531865466)全部通过：参考镜像 JavaScript 返回的 10 个 `[name, bookUrl]` 与 Reader `webView=true` 搜索结果的相同投影 SHA-256 均为 `46c56e39a396824f6cc2d7ff920791f5f6810419c3d60d2e271740d7817ce561`；Reader HTTP 200、`isSuccess=true`、`errorMsg=""`；采样相隔 237.914 秒。制品明确标注 `projectionEqual=true`、`fullResponseParityProven=false`、`originalJarCompared=false`、`referenceProvenProduction=false`。这是比“都是 10 本”更强的公开样本证据，但仍缺原 JAR、原生产远程实例、真实登录书源和同一 HTTP 响应内容控制，不能称三方兼容验收完成。

## 2026-10-03 当前候选公开页复验

提交 `024da9d7` 的[手动配对作业 37090342192](https://github.com/warpdotsys/reader-dev/actions/runs/37090342192)已完成旧参考镜像、完整 Reader/Camoufox 镜像及比较作业。下载的[独立 JSON 证据](evidence/public-webview-024da9d7-2026-10-03.json)记录两侧均为 HTTP 200、10 项书目，书名/URL 投影 SHA-256 均为 `66b358fd579636fea1922725c1c228e318a554f98a989c3e881e80619a9beec4`；Reader 为 `isSuccess=true`、空错误信息。两侧观测相隔 `416.662` 秒，旧参考原始页面摘要为 `018b3d6351518a8744b047eb487a461426d888275d8da2d3f89a959b8bd3e1bd`。

同一完整镜像还通过合成 Cookie 隔离、脚本书名改写、目标 POST 和四用户同时请求。cgroup 内存峰值 `832761856` 字节（约 794 MiB），PIDs 任务数（含线程）峰值 199；2 GiB / 256 PIDs / 2 CPU 限制下，OOM 和 PIDs 限额事件均为 0，最终 swap 使用为 0。这只是短时合成加公开页负载，不代表真实登录站点或生产长期吞吐。

当前投影摘要与 2026-09-29 的摘要不同，说明公开列表随时间变化；本轮只以两侧同一时段的相同投影作有限比较。原 JAR 没有加入本次托管公开页配对，固定旧镜像也未证明是原生产实例；完整响应等价继续标记为未证明。此前本地原 JAR 的静态目录结果不能成为本轮的第三侧。

## 2026-10-05 当前导航修复源码的同轮公开配对

源码 `bbfcda53d379f4d2a992ed8cc092b9e07fcbcfe6` 的[手动配对 37309242432](https://github.com/warpdotsys/reader-dev/actions/runs/37309242432)已全部完成，三条实际运行作业均为 GitHub hosted runner。下载的[独立四份小报告与散列](evidence/public-webview-bbfcda53-2026-10-05.json)核对通过：固定历史参考镜像直接页面/投影 HTTP 均 200，内置默认引擎经隔离 Reader 为 HTTP 200、`isSuccess=true`、空 `errorMsg`；两侧各 10 本、书名/URL 投影 SHA-256 均 `3b131db566c96a9a06bd0d7f576a10321344da8e49221f4b2ef975d9c9ed91d8`，采样相隔 425.668 秒。不是相同上游响应控制，原 JAR 未加入、参考镜像未证明是原生产实例，完整响应等价仍为 false。

同作业的默认浏览器 14 项也实际零跳过/失败/错误，严格守卫接受，XML SHA-256 `5f8889869c49791d5287fefb68d893ebf413937f070cecb57e4339dd5076378e`；合成 GET/POST/脚本、Cookie 隔离及四账号并发样本通过。公开负载后的单镜像短时内存峰值 `858886144` 字节（约 819 MiB）、PID 峰值 200；2 CPU / 2 GiB / 256 PID 下触限/OOM 事件和实际 swap 为 0，仍配置允许 1 GiB swap。这不是持续负载容量保证。

未向 GitHub 传起点凭据、原始 JAR、私人正文或生产存储，也没有改生产。起点网站公开首章可见是另一个仅浏览器 UI 观察，不代替此处 Reader 列表解析；反过来此配对也不能关闭起点精确搜索/认证/免费正文与真实三方缺口。

## 尚未验证

- 原 JAR／恢复版的一致性样本是公开静态目录的真实 HTTP/规则解析，未启用 `webView`；另一个容器内样本启用了内置 Camoufox，但没有同条件运行原 JAR 和旧远程 WebView。两项证据不能拼接成三方等价，且站点本身不是依赖 JavaScript 或登录态的样本。
- 还需覆盖真实脚本、重定向、登录态 Cookie、跨用户隔离、超时、异常响应、来源限制与并发资源。在生产主机上，旧 Reader 配置引用的远程 WebView 服务当前不可用；恢复其可复现对照前，不应声称完成三方验证。
- 页面可能随站点更新而变化。脚本每次都记录前后上游摘要；如果摘要变动，即便两个 JAR 的结果不同也须先排除内容漂移。测试成功不授权持续高频抓取该站。
