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

## 已知问题／尚未验证

1. 原 JAR／旧远程／新默认引擎的 Promise 同条件三方尚未运行。既有三方的同步脚本结果不自动证明新增异步结果。
2. 起点匿名详情此前两次实际默认 Camoufox 请求仍是成功壳／空期望元数据。该探针未使用 `webJs`，本缺陷不能被认定为它的唯一原因，更不能宣称已修起点登录／正文。
3. 真实认证、全部 UI／阅读格式、当前新源码长期负载及生产用户可见验收仍未完成。
4. 生成 async Cookie 不等于真实网站登录 Cookie 已验证；未读取新 Cookie、未请求真实正文、未安装待授权 uidmap、未改用户 Reader 或生产数据。
5. 本机 Windows build 只验证编译和打包；缺少最终 Linux 默认浏览器时，明确保留环境跳过，不伪造空实现。

## 回滚

本修改尚未合并或正式发布。可在新的维护提交中恢复修改前的 worker，并同步撤回第 15 项及门禁变更；必须重新构建／验收，不在运行中的 JAR 内热替换脚本。保留旧失败和新验收证据，不覆盖工作区未提交文件、不直接 reset。仅返回旧 14 项不表示异步缺陷消失。
