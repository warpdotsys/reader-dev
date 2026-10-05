# Chromium 内容导航竞争修复候选

更新日期：2026-10-05。源码基线 `6f56c990`；本增量已本机验证，自己的 GitHub 托管结果尚待。仅 opt-in `LocalWebviewRenderer`，不宣称默认 Camoufox 或真实起点已经修好。

## 已实际复现，不是近似源码推断

前一轮无凭据 Reader 起点搜索实际命中 Playwright 的 `Unable to retrieve content because the page is navigating and changing the content.`。另建纯生成站点：DOMContentLoaded 后立即进入有限导航链，12 次真实 Chrome 内容读取全部同样失败；导航阶段失败、其他错误及网络拒绝均 0。没有外站或凭据。新 JUnit 回归在旧渲染器下实际 1 项 / 1 失败 / 0 跳过，11.425 秒，原 XML 冻结保留。

对应[固定 Playwright 1.63.0 源码](https://github.com/microsoft/playwright/blob/v1.63.0/packages/playwright-core/src/server/frames.ts)确实会在读取 HTML 的上下文失效后报告这一错误。[官方 load-state 文档](https://playwright.dev/java/docs/api/class-page#page-wait-for-load-state)说明已达到状态的旧文档可以立即返回，因此不能只追加一次不带预算的等待。

## 可读源码修复与边界

`BrowserDocumentSnapshot` 只重读没有用户脚本的 HTML 快照；只匹配上述已验证 Playwright 错误，明确不吞操作超时、页面关闭和其他异常。每轮先检查网络拒绝，短暂泵送事件，再以同一递减预算等待 DOMContentLoaded；不等 networkidle、不重发导航或 POST、不重执行书源脚本，也不放开 DNS/私网/Host 或导入凭据。它是快照阶段的单一重试预算，并非整个渲染的 OS 硬墙钟抢占，未修复任意 JavaScript 无限运行。

生成导航回归连续三轮读到最终页面，真实服务端计数确认三个初始 POST 各发送一次。11 项非门控单元覆盖成功不等待、递减预算、持续导航超时、网络拒绝优先以及非目标错误不重试。默认 Camoufox worker 没有修改，该引擎的类似导航竞争仍须单独实证。

## 已成功重建和本机验证

- 真实 Chromium 11 + launch policy 1 + 快照单元 11，共 23 项，0 失败/错误/跳过。
- 完整 Gradle 40 套 / 143 项，实际执行 131 项，0 失败/错误；12 项 Camoufox 因本机运行时未配置门控跳过，不算实跑通过。
- 完整 Vue 3 JAR SHA-256：`3ac63d9c126663d536a0a7a67cbca91fc4f6243699345a1a1b4d1e299e1ac597`。81 个包内 UI 文件逐字节匹配 dist，新快照类实际存在。
- 原 JAR 散列仍为 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`，没有覆盖或在本轮重新执行原件。47 项发布链路测试及 actionlint 通过；未运行 shellcheck/pyflakes。

在工程根目录，已有工程 JDK 11、Wrapper、依赖缓存、Vue 3 dist 和实际 Chrome 时，可重复定向命令（环境仅当前终端）：

```powershell
$env:JAVA_HOME=(Resolve-Path .tools/jdk-11.0.8).Path
$env:GRADLE_USER_HOME=(Resolve-Path .gradle-user-home).Path
$env:JAVA_TOOL_OPTIONS='-XX:ActiveProcessorCount=2 -Dfile.encoding=UTF-8'
$env:PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD='1'
$env:READER_BROWSER_EXECUTABLE='C:\Program Files\Google\Chrome\Application\chrome.exe'
.\gradlew.bat '-PreaderWebUi=vue3' test --tests 'com.htmake.reader.utils.LocalWebviewRendererTest' --tests 'com.htmake.reader.utils.ChromiumLaunchPolicyTest' --tests 'com.htmake.reader.utils.BrowserDocumentSnapshotTest' bootJar --offline --no-daemon --max-workers=2 '-Pkotlin.compiler.execution.strategy=in-process' '-Dorg.gradle.jvmargs=-Xmx1536m -XX:ActiveProcessorCount=2'
```

初次基线命令引用错误导致 Gradle 未执行测试；修正后才取得上述真实失败。诊断工具起初误将单元素 JSON 数组当对象，已修正且不再输出原数据；它是工具错误，不是站点失败。

## 起点匿名复测仍不构成搜索验收

用原样无凭据官方桌面书源、两个新隔离存储/生成账号、此新 JAR 做真实 Reader 请求。保持原九主机 DNS 快照的两次结果为 HTTP 200 / `isSuccess=false` / `data=null`，导航竞争分类为 false，新拒绝为 `DNS_FAILURE` / 指纹 `5fdb41b4266676bf`。

离线对 47 个公开候选主机散列核对，确认是 `ssl.captcha.qq.com`；Google DoH 实际 A 答复为公网 `157.255.220.168`，TTL 300。另一解析服务未给出预期 JSON 结构，不当作第二份验证。只在新测试进程的 hosts 快照追加此一公网静态主机，系统 hosts、生产解析、全局安全策略及原九主机证据未变。旧/短 TTL DNS 快照并非长期站点配置。

补充快照的真实 Reader 返回 HTTP 200 / `isSuccess=true` / `errorMsg=""` / `data=[]`，8.209 秒，但精确书名仍为 0，**不是成功搜索**。另一次新匿名浏览器仅拟检查页面结构，因指纹 `ad4a9c600a0e9679` 的不同未记录 DNS 请求被安全门禁中止，未获得 HTML。不能据此确定空结果是旧规则、验证码还是其他页面结构问题；没有处理/绕过验证码，也没有购买或请求章节。

两个 Reader Cookie 前后均 0，自有 PID 59112 / 49588 持有句柄关闭、18944 无监听，用户 18931/PID 61244 保留。没有重用已清除真实 Cookie，没有读取其他本地正文，没有上传正文/凭据；未逐 PID 独立审计所有 Windows 后代，不冒充完整回收证明。[失败、散列、测试、匿名结果及限制](evidence/browser-navigation-2026-10-05.json)。

## 托管门禁、尚未验证和回滚

托管 Chromium 步骤新增快照单元，强制三套实际 XML 计数 11/1/11 且 0 失败/错误/跳过，在下一 Gradle 调用前冻结并保留失败证据。前一后台服务修复 `6f56c990` 已完成四工作流、23 项真实浏览器/策略报告及 22 份原生/完整镜像 JSON 审核，详见[对应报告](CHROMIUM-BACKGROUND-NETWORK-2026-10-05.md#后台服务修复已托管复验源码-6f56c990)；它们不证明本次后续源码。

本增量仍待自己的托管验证，起点准确搜索、认证三方、正文、默认引擎导航竞争、最终本机非 root 镜像和生产验收均未完成。没有合并默认分支、正式发版、上传注册表或部署。回滚在新的独立干净工作树撤回本源码增量后重建；旧内容读取错误会恢复。不要 reset/checkout 覆盖用户脏目录；无字节码补丁或数据迁移。
