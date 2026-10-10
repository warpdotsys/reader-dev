# Camoufox 异步书源脚本结果恢复

日期：2026-10-06。本文区分语言层实证、可读源码修复和实际浏览器验收，不等于起点登录或生产发布通过。

## 旧缺陷已实际复现

在修改前，用 Python AST 提取当前 `worker.py` 真正传给 `page.evaluate` 的包装函数，再用 Node 24.19.0 实际执行。输入 `Promise.resolve({answer:42,ready:true})` 返回 `{}`，不是期望的 `{"answer":42,"ready":true}`。旧包装在 Promise 完成前调用 `JSON.stringify`，丢弃异步返回值。

这不是伪造业务返回值，也不是原 JAR／旧远程 WebView 的运行证据。原始源码和函数 SHA-256、实际小输出见[复现记录](evidence/camoufox-async-script-language-2026-10-06.json)。

[Playwright 官方说明](https://playwright.dev/python/docs/evaluating)指出，直接返回 Promise 时会等待其完成；旧包装提前转成字符串，已经不再向 Playwright 返回该 Promise。官方说明不是原 JAR 的兼容实证。

## 可读源码修改

- 只修改包内 Python worker 的普通 `webJs`／JavaScript 结果路径，不增加宿主 Chrome、外置服务、依赖或字节码补丁。
- 原脚本只执行一次。用原生 Promise adoption 得到完成值后，字符串保持原样，其余值按 JSON 格式输出；null/undefined 仍为空字符串。thenable getter 不预读，避免重复触发其副作用。
- 每次请求使用随机、不可枚举的页面临时结果槽。Python 用单个单调时钟预算读取状态，等待期间持续检查网络拒绝；不依赖网页计时器实现超时，不重新导航／重发 POST／重执行脚本。
- 拒绝只返回 `SourceScriptRejected` 类名，不复制脚本异常文本、返回的异常对象、URL 或凭据。预算耗尽返回 `SourceScriptTimeout`。替换文档丢失状态时失败，不猜测安全重放。
- 尽力删除临时槽，最终由既有独立 context 关闭收回状态。4 MiB 正文／8 MiB 协议和 Cookie 限额、私网安全门禁保持不变。
- `sourceRegex` 是“捕获资源 URL”的另一种契约，继续忽略脚本返回值，本次不把它改成等待 Promise。

预算是脚本阶段的一个预算，不是包括浏览器启动在内的总时限。同步死循环、浏览器事件线程阻塞或传输挂死仍依赖父进程 `2 × timeoutMs + 15 秒` 紧急看门狗；不能声称 Python 能抢占所有 JavaScript。

## 本机验证与门禁

- 旧缺陷的实际语言执行证据已保留。
- 新 JS 夹具直接执行 worker 内的三个原文函数，不用复制一份替代实现；26 项包括同步／异步结构值、延迟、thenable、拒绝、循环对象、永久 pending、单次副作用及临时状态清理。这是 Node 语言验证，不是 Firefox 验收。
- Python 控制时钟夹具覆盖单预算、启动／读取耗时、状态丢失、网络拒绝、未知传输异常、清理失败、UTF-8 限额及不重放。这些夹具不执行真正浏览器。
- 第一轮 142 项 Python 中只有新 TAP 解析检查失败：Windows 的 Node 默认使用另一种显示格式，实际 25 项 JS 均通过。明确指定 TAP reporter 后复验通过；之后增加 getter 单次 adoption 的第 26 项，须以最终报告为准。不把夹具解析失败算作产品浏览器失败。
- 实际 Camoufox JUnit 新增第 15 项：生成脚本拒绝／永久 pending 后必须得到 worker 级错误、下一次真实渲染恢复，并以服务端计数确认原导航、表单 POST、脚本 POST 不重复。已有结构返回值用例增加 Promise 字符串／对象／数组／数字／null、异步 DOM 和真实生成 Cookie 及命名空间检查。
- 共同验收守卫严格要求 15 项、零失败／错误／跳过和准确名称；旧 14 项成功 XML 不能证明新增行为。真实执行由 GitHub 托管 runner 完成，不在本机下载大镜像。本机环境门控跳过不算实际 Camoufox 通过。

可重复本机纯检查：

```powershell
$env:READER_TEST_NODE = 'C:\Users\chong\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe'
& 'C:\Users\chong\anaconda3\python.exe' -m unittest discover -s src/test/python -p '*_test.py'
& $env:READER_TEST_NODE --test --test-reporter=tap src/test/javascript/camoufox-source-script.test.mjs
```

CI 使用已固定的 Node 24、JDK 11、Camoufox 依赖和原生 runner；构建、真实测试、包内 worker 身份守卫必须先通过，才允许导出共享 JAR／镜像。当前托管复验尚待本修改提交后的独立运行，不能拿旧镜像或其他工作流绿灯替代。

### 最终本机记录

最终语言用例 26 实际通过；Python 142 项中 141 执行／1 个 Windows 符号链接跳过、零失败／错误，发布结构／字节守卫 54 全执行通过。Java/Kotlin 的 41 份 XML 共 149 项：123 执行／26 环境跳过、零失败／错误；其中全部 15 项 Camoufox 本机跳过，不能算真正浏览器验收。

首次打包期间源码继续完善了原生 Promise adoption，逐字节核对抓到包内 worker 落后一版。这份中间包不作为通过产物。再次执行 Gradle `processResources bootJar` 成功，最终 `reader-4.0.7.jar` SHA-256 为 `7e05f00f75a95870ec6c2d5454f7d4ce0cbdfda49724a6dfdb64112902b57269`，包内／最终源码 worker 哈希均为 `a1a2a1c5d6aa445e24e5a08c7dcd579831255f19bf1bd9c472ecfdab711f48c9`。这是本机包，不等于后续托管产物的相同字节。58 份已有报告前后哈希全部相同。

[完整本机计数、41 份 XML 摘要及拒绝的中间包](evidence/camoufox-async-script-local-build-2026-10-06.json)分别保留，不追认旧包，不计算环境跳过为执行成功。此次没有启动原始 JAR、用户 Reader、真实 Cookie／正文或生产服务。

本机最终源码的重复构建命令（项目根目录，保留既有 build 数据，不执行 clean）：

```powershell
$readerProject = (Get-Location).Path
$env:JAVA_HOME = Join-Path $readerProject '.tools\jdk-11.0.8'
$env:GRADLE_USER_HOME = Join-Path $readerProject '.gradle-user-home'
$env:PATH = "$env:JAVA_HOME\bin;C:\Users\chong\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin;" + $env:PATH
& .\gradlew.bat -PreaderWebUi=vue3 test bootJar --max-workers=2 --no-daemon
```

### 已发起 GitHub 托管 runner 复验（当时状态）

源码提交 `0664dc7ba31e52a97a604cb7df386840cb256b82` 已推送；PR 56 仍未合并。GitHub API 独立确认被测合并快照 `2b94a11b49c049f4a762a624860b33e0adab018a`，它不是默认分支提交。实际运行：

- [Java/Kotlin 37422265010](https://github.com/warpdotsys/reader-dev/actions/runs/37422265010)已成功；下载 30,646 B 小制品，41 份 XML 的计数与 testcase 一一核对，149 项中 123 执行／26 跳过、零失败／错误。15 项默认 Camoufox 在这条普通 CI 全部跳过，不算浏览器通过。
- [默认浏览器／完整镜像 37422265049](https://github.com/warpdotsys/reader-dev/actions/runs/37422265049)、[原生发布演练 37422265057](https://github.com/warpdotsys/reader-dev/actions/runs/37422265057)、[Vue 3 37422265116](https://github.com/warpdotsys/reader-dev/actions/runs/37422265116)已实际发起，尚未据此验收终态。默认引擎的真实 15 项 XML、包内身份和双架构后续结果仍须独立核对。
- 本机真实生成的“15 项全部环境跳过”XML 也实际被共同发布守卫拒绝，退出码 1；不只是合成 XML 的负向单测。

只有小型证据下载到本机，没有下载新镜像／共享 JAR。上一个 `929d518d` 的四条成功仍是旧源码结果，不能替代本次异步修复。

### 新增真实默认引擎已独立核对（消费端完成前）

本修改的两份实际 Camoufox XML 已下载并再次通过共同守卫：普通完整镜像与共享发布 JAR 两次各 15 项／零跳过、失败、错误。新增拒绝／永久 pending／下一渲染恢复契约分别实际用时 13.018／14.553 秒；结构结果／延迟 DOM／生成 Cookie／单次 POST 用例分别 22.416／28.488 秒。这是两次独立托管执行，不是把本机 15 项跳过改成通过。

Java/Kotlin、Vue 3、完整镜像三条工作流已成功；Native 的共享 JAR、两架构原生生产者也成功，但消费者重载／导入和最终汇总仍待完成。当前核对 108 份 XML 和两份完整镜像 JSON：普通 JVM 两套各 149 项中 123 执行／26 环境跳过；实际 Camoufox 各 15、Chromium／快照 23、Vue 3 21 均零跳过／失败／错误。未把五个手动专项作业的 skip 算作成功，未宣称本次截图目视验收。

完整镜像生成请求、四用户 Cookie 与资源预算通过；短时峰值 `825356288 B`（约 787 MiB），PID 峰值 207，2 CPU／2 GiB，触限、OOM 和实际 swap 为 0，**但仍允许 1 GiB swap**，不是长期无泄漏保证。这里只下载小型报告，不下载新共享 JAR 或多 GiB 镜像。

[108 份实际 XML、两份 JSON、准确制品元数据和剩余范围](evidence/camoufox-async-script-hosted-2026-10-06.json)保留逐文件哈希。归档 ZIP 摘要是 GitHub 声明值，下载解压后的文件哈希才是在本机重新计算的；两者不混写。这一后端增量不等于原 JAR／旧远程 Promise 三方、起点真实登录／正文或生产交付。

### 双架构消费端完成后的最终核对

`37422265057` 最终全部六作业成功。实际 GitHub 托管 runner 分别完成共享 JAR、原生 AMD64／ARM64、两个新 runner 上的重载运行、以及同一发布导入器消费两份已测试镜像；没有 registry 登录／推送或生产访问。此前“消费端尚待完成”是历史中间状态，现已由本次自己的实际证据关闭。

下载的 7,072 B 消费端制品含 20 份 JSON，逐文件哈希及严格类型／字段独立核对：两架构原生与重载的实际运行 JAR SHA-256 均为 `32db24dac1dcf4b56dd13ccea1d75dafd1f5450f918873ec1a33bfb8d7bd40bd`；版本对象都是 `4.0.7`／合并快照 `2b94a11b...`，不是 `true` 占位。重载 image ID 分别与自己的原生 metadata 相同；完整 HTTP／ReturnData、GET／POST、四用户 Cookie、并发生成搜索及原始资源字段通过。

四次原生／重载的最高短时内存为 `864952320 B`（约 825 MiB），最高 PID 202，全部 2 CPU／2 GiB／256 PID，触限／OOM／实际 swap 为 0；**仍允许 1 GiB swap**。加上完整镜像的两份 JSON，目前共核对 108 份 XML／22 份 JSON。多 GiB 镜像和共享 JAR 只由 GitHub runner 下载；本机只下载小型证据与生成截图，不把未目视的八张管理图作为新视觉验收。

本阶段的四条 GitHub 托管工作流／10 个实际作业已通过，不代表 PR 合并、正式标签、registry 发布、认证三方或生产上线。新 JSON 已补在[同一实际托管证据](evidence/camoufox-async-script-hosted-2026-10-06.json)，旧局部状态和首次打包不匹配证据保留。运行后补充文档与后续真实 Reader API 验收提交一起保存，避免只为文档更新重复触发大镜像构建。

## 已知问题／尚未验证

1. 原 JAR／旧远程／新默认引擎的 Promise 同条件三方尚未运行。既有三方的同步脚本结果不自动证明新增异步结果。
2. 起点匿名详情此前两次实际默认 Camoufox 请求仍是成功壳／空期望元数据。该探针未使用 `webJs`，本缺陷不能被认定为它的唯一原因，更不能宣称已修起点登录／正文。
3. 真实认证、全部 UI／阅读格式、当前新源码长期负载及生产用户可见验收仍未完成。
4. 生成 async Cookie 不等于真实网站登录 Cookie 已验证；未读取新 Cookie、未请求真实正文、未安装待授权 uidmap、未改用户 Reader 或生产数据。
5. 本机 Windows build 只验证编译和打包；缺少最终 Linux 默认浏览器时，明确保留环境跳过，不伪造空实现。

## 回滚

本修改尚未合并或正式发布。可在新的维护提交中恢复修改前的 worker，并同步撤回第 15 项及门禁变更；必须重新构建／验收，不在运行中的 JAR 内热替换脚本。保留旧失败和新验收证据，不覆盖工作区未提交文件、不直接 reset。仅返回旧 14 项不表示异步缺陷消失。
