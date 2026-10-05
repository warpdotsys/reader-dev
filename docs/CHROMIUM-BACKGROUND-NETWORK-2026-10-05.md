# Chromium 基线的后台请求误判

## 已从恢复源码与隔离运行验证

先离线比对公开候选域名，指纹 `191347bfe55d0ca9` 对应 `www.google.com`。不能仅凭指纹认定来源，因此在原运行时代码上新增真实 Chrome 生成页面回归：只允许生成夹具解析，所有其他域名在 DNS/外连前拒绝。没有访问起点或导入凭据，仍实际触发相同 `DNS_FAILURE`，1 项测试失败、0 跳过，原 XML 冻结。

生成探针使用独立临时 Chrome profile、自有出口代理及没有 Cookie 的生成页面，未读取用户 Chrome profile。实际 Chrome 为 `154.0.8037.58`。只关闭默认搜索预连接后仍失败；生成 NetLog 中有两次浏览器自身的 AI 模式资格请求。只关闭 AIM 后，请求消失但仍有四次默认搜索预连接。添加 automation 参数也未修复。组合关闭后，两类事件均为 0、生成页面正常渲染。完整 NetLog 留在忽略的本机测试目录，不上传其原始内容。

此定位与 Chromium 的[对应版本 AIM 服务源码](https://github.com/chromium/chromium/blob/154.0.8037.58/components/omnibox/browser/aim_eligibility_service.cc)及[默认搜索预连接源码](https://github.com/chromium/chromium/blob/154.0.8037.58/chrome/browser/navigation_predictor/search_engine_preconnector.cc)相符；网站来源不是单凭这些外部源码推断的。它解释匿名基线的一条失败路径，不能把它直接套回先前真实认证的两轮测试。

## 源码修正和安全边界

只调整 opt-in `LocalWebviewRenderer`：禁用 `PreconnectToSearch`、`PreconnectFromKeyedService`、`AimEnabled` 三个浏览器自身服务。覆盖 feature 参数时完整保留固定 Playwright 1.63.0 的 15 个默认禁用项；新的非门控单元测试直接读取实际 driver bundle，升级后默认项不一致必须失败并人工复核，不能悄悄丢掉原来的限制。

没有放行 Google 域名、Fake-IP 或私网，没有吞掉出口拒绝，没有改系统 DNS/hosts、生产代理、页面请求、Cookie、存储、ReturnData 或默认 Camoufox 引擎。Google 是页面合法资源时仍按原网络策略处理，不作全域例外。不是指纹或反爬能力声明，也不是 Camoufox 已通过起点认证。

## 已成功重建和本机复测

组合修正后 9 套件、31 项定向测试全部通过，0 跳过；真实 Chromium 10 项包含 GET/POST、Cookie 命名空间、脚本子资源、SSE、sourceRegex、编码、私网/重定向和新后台请求回归。完整后端为 39 套件、131 项、0 失败/错误；12 项真实 Camoufox 因本机运行时未配置而跳过，不能计为已执行。

完整 Vue 3 JAR 已重建，81 个包内 UI 文件与当前 dist 逐字节核对一致。SHA-256 为 `ef8cd41cd5844d2b6993104ec42dad5ce0a637bd1d04295b65174e495883ccae`。原 `D:/Download/reader-pro-3.2.14.jar` 的散列仍为 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`；本轮没有重新运行原件作为差分对照。47 项发布链路测试及 actionlint 通过，未运行 shellcheck/pyflakes。早先命令参数顺序错误与只禁用预连接的失败分别保留，不称为站点问题或成功。

随后用原样、无凭据的官方桌面书源，在三套新隔离存储/生成 Reader 账号中实际搜索共五次；只复用同一九主机进程级 DNS 快照，未增加 Google 或其他解析例外。原 Google 指纹拒绝消失，但 **五次仍 HTTP 200 / `isSuccess=false` / `data=null` / 精确书名 0**。进一步脱敏分类实际确认 `Page.content()` 的页面导航竞争：页面仍在跳转/改变内容，无法读取快照。它是新定位的后续故障，**尚未修复**，不能将搜索宣称为通过。

没有请求章节正文、购买、使用网站账号或重用已清除 Cookie。三套 Cookie 前后均为 0；持有句柄关闭自有 JVM PID 65104 / 66528 / 30972 后，18944 已关闭，用户 18931/PID 61244 监听保留。没有独立逐 PID 枚举 Windows 后代，不冒充完整进程回收审计。[失败、生成控制、本机测试与匿名结果](evidence/chromium-background-2026-10-05.json)。

可重复定向命令（工程 JDK 11、Wrapper、依赖缓存与测试 Chrome 已准备）：

```text
gradlew.bat -PreaderWebUi=vue3 test --tests com.htmake.reader.utils.LocalWebviewRendererTest --tests com.htmake.reader.utils.ChromiumLaunchPolicyTest bootJar --offline --no-daemon --max-workers=2
```

`--tests` 属于 `test`，必须放在下一 `bootJar` 任务之前。真实 Chromium 合约需要 `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1` 和 `READER_BROWSER_EXECUTABLE`；只运行 Gradle 不配置环境会门控跳过，不等于通过。

## 尚未验证、CI 与回滚

GitHub 托管集成新增两份实际 XML 的强校验：Chromium 10 项、launch policy 1 项，全部必须 0 跳过/错误/失败，并在下一次 Gradle 前冻结、上传小型证据。新增源码/单元路径也进入两类触发器。新提交仍需自己的托管验收；此前 `6d398926` 的四条工作流及双架构持续测试不能用于证明本修改。

页面导航竞争、Reader 起点搜索/正文、真实认证三方、最终本机非 root 镜像及生产验收仍待完成。当前没有正式发版、合并默认分支或部署候选。回滚可在独立干净工作树撤回本次可读源码增量并重建；恢复旧 Chrome 基线也会恢复此已知误判。不得对用户脏目录 reset/checkout，没有字节码补丁或数据迁移。
