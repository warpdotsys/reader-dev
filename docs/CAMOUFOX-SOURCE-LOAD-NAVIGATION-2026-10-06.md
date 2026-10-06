# 源规则执行时机：旧引擎 load 与候选 DOMContentLoaded

## 已验证事实

`2f5b2289` 的新镜像匿名专项实际得到 `sourceStateMissing`，规则等待期间观察到主框架导航，未取到快照；依然不是唯一原因或验证码证明。[实际九份报告与范围](evidence/public-metadata-source-state-diagnostics-hosted-2026-10-06.json)。

为确定兼容性依据，只读请求固定旧远程参考镜像的 OCI index、AMD64 manifest 和两个小 layer，不启动镜像、不完整下载／解包。逐层 SHA-256 均实际匹配：2,431 B 应用 layer 内 `app/index.js` 的 `loadOptions` 明确设置 `waitUntil: load`；四处 goto／setContent 等待同一选项，再在 117／128 行 await 执行源脚本。package.json 只声明 Playwright `^1.25.1`，不能称已验证安装了准确版本。未把公开令牌、完整源码或用户数据存入仓库。[可追溯静态证据](evidence/archived-source-load-semantics-2026-10-06.json)。这仍不是原生产实例或原 JAR 的同输入行为。

当前候选 worker 对全部首次 goto／setContent 使用 `domcontentloaded`，随后单次执行规则。与旧参考源码存在明确执行时机差异；是否解释起点实站故障仍待验证。Playwright 区分 [DOMContentLoaded 和 load 等完成条件](https://playwright.dev/python/docs/api/class-page#page-goto)，不能把 DOM 已就绪当成所有载入都完成。

## 新增的受控回归：先不修改产品

`generatedSourceRuleRunsOnceInTheDocumentThatCompletesLoad` 使用纯生成 HTTP 页面：DOM 已就绪后通知服务端，真实未完成图片响应阻止 load；服务端确认图片请求与 DOM 通知后触发一次主文档跳转，最终请求记录原文档 readyState 必须为 interactive。短延时只用于调度，是否处于 DOM-ready／未-load 由握手及状态明确断言，不靠计时推断。

源规则有一个可观察生成 POST，并返回延迟 Promise。要求它只在完成 load 的最终文档中执行一次，返回最终标记；原始导航／原始 POST／最终导航／规则副作用均恰好一次，下一次独立上下文健康。提前执行时可能丢失状态或在中间文档产生副作用，两者均失败，不以重跑规则来满足计数。

精确具名门禁增至 18 项，旧 17 项 XML 不证明此用例；新增合成 XML 拒绝测试保留全跳过／伪计数／缺项等原守卫。此阶段尚未修改 worker，也未宣称新的真实用例已通过或已复现；本机只能编译测试，真实红灯基线须由本提交的托管运行取得。`2f5b2289` 的两份真实 17 项、界面 18＋1 与双架构导入证据仍只属于原候选，不能回填为本新增验收。

本地全量 Python 193 项中 192 实际通过、1 个 Windows 环境跳过，0 失败／错误；这是生成数据与门禁单元测试，不是新增真实浏览器的绿灯。JDK 11 离线 `compileTestKotlin` 实际成功，限制 2 个活动处理器／512 MiB／单 Gradle worker；只是编译，未在本机启动真实引擎、用户 Reader 或原 JAR。

## 后续实现的约束

只有受控基线和调用时机依据成立后，才评估将有显式规则、无 sourceRegex 的载入阶段对齐 load；无脚本快照／sourceRegex 另有既存行为，不能同时改掉。不得加入额外 goto、源脚本重执行、POST 重发、空结果成功、任意静默错误恢复或扩大预算／私网放行。源脚本执行后再发起导航仍须保持失败，不能用此修复承诺跨文档继续执行任意规则。

产品修复、实站新候选、原 JAR／旧引擎同输入以及长期／生产均尚未由此完成。原失败与诊断证据继续保留。回滚本阶段仅撤回生成回归和 18 项门禁，不能因此把旧实站失败称为解决。
