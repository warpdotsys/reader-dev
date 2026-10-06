# 源规则执行时机：旧引擎 load 与候选 DOMContentLoaded

## 已验证事实

`2f5b2289` 的新镜像匿名专项实际得到 `sourceStateMissing`，规则等待期间观察到主框架导航，未取到快照；依然不是唯一原因或验证码证明。[实际九份报告与范围](evidence/public-metadata-source-state-diagnostics-hosted-2026-10-06.json)。

为确定兼容性依据，只读请求固定旧远程参考镜像的 OCI index、AMD64 manifest 和两个小 layer，不启动镜像、不完整下载／解包。逐层 SHA-256 均实际匹配：2,431 B 应用 layer 内 `app/index.js` 的 `loadOptions` 明确设置 `waitUntil: load`；四处 goto／setContent 等待同一选项，再在 117／128 行 await 执行源脚本。package.json 只声明 Playwright `^1.25.1`，不能称已验证安装了准确版本。未把公开令牌、完整源码或用户数据存入仓库。[可追溯静态证据](evidence/archived-source-load-semantics-2026-10-06.json)。这仍不是原生产实例或原 JAR 的同输入行为。

修复前候选 worker 对全部首次 goto／setContent 使用 `domcontentloaded`，随后单次执行规则。与旧参考源码存在明确执行时机差异；是否解释起点实站故障仍待验证。Playwright 区分 [DOMContentLoaded 和 load 等完成条件](https://playwright.dev/python/docs/api/class-page#page-goto)，不能把 DOM 已就绪当成所有载入都完成。

## 新增受控回归与实际红灯基线

`generatedSourceRuleRunsOnceInTheDocumentThatCompletesLoad` 使用纯生成 HTTP 页面：DOM 已就绪后通知服务端，真实未完成图片响应阻止 load；服务端确认图片请求与 DOM 通知后触发一次主文档跳转，最终请求记录原文档 readyState 必须为 interactive。短延时只用于调度，是否处于 DOM-ready／未-load 由握手及状态明确断言，不靠计时推断。

源规则有一个可观察生成 POST，并返回延迟 Promise。要求它只在完成 load 的最终文档中执行一次，返回最终标记；原始导航／原始 POST／最终导航／规则副作用均恰好一次，下一次独立上下文健康。提前执行时可能丢失状态或在中间文档产生副作用，两者均失败，不以重跑规则来满足计数。

精确具名门禁增至 18 项，旧 17 项 XML 不证明此用例；新增合成 XML 拒绝测试保留全跳过／伪计数／缺项等原守卫。`993d5ef6` 不改产品 worker，仅建立受控基线；`2f5b2289` 的旧绿灯不能回填为新增验收。

本地全量 Python 193 项中 192 实际通过、1 个 Windows 环境跳过，0 失败／错误；这是生成数据与门禁单元测试，不是新增真实浏览器的绿灯。JDK 11 离线 `compileTestKotlin` 实际成功，限制 2 个活动处理器／512 MiB／单 Gradle worker；只是编译，未在本机启动真实引擎、用户 Reader 或原 JAR。

两个自身托管运行已经实际终止：Native `37454361359` 为 18 项／2 失败，完整镜像 `37454361323` 为 18 项／1 失败，均 0 错误／跳过。新的规则时机用例两侧都得到 `SourceScriptStateLost`；原导航、原 POST、最终导航、规则 POST、阻塞资源均 1 次，原文档跳转时 interactive，夹具超时 0。两份 XML 散列独立记录，原精确 18 项守卫实际执行均 exit 1。没有把失败吞掉或只根据工作流状态推断。[实际红灯证据](evidence/source-load-navigation-red-hosted-2026-10-06.json)。

Native 的另一处失败是旧早期导航夹具要求“服务端收到最终请求时，DOM-ready beacon 收到数必须为 0”，实际为 1；同源码完整镜像该项通过。其后 loading 断言未执行，不能宣称它已证明跳转时 DOM 就绪，也不能唯一归因为浏览器或产品。两个独立 HTTP 请求的到达顺序不是客户端事件发生顺序，因此夹具改为同一 JS turn 在 `location.replace` 调用前记录 readyState 和 DOM 事件计数，要求 loading 与计数 0，仍保留真实阻塞／一次原 POST／最终文档／Cookie 断言。原服务端计数保留诊断，不用重试掩盖；这属于观测方式修正，仍需自己的真实复验。

## 有界产品修复：自己的托管验收尚待完成

受控基线和旧源码依据成立后，仅有显式非空 javaScript 且无 sourceRegex 的现有 goto／setContent 调用改为 `wait_until=load`，不新增调用。无脚本 HTML 快照与 sourceRegex 仍使用 DOMContentLoaded；超时值、父进程预算、网络拦截、Cookie 回写和单次规则执行不改。

[Playwright 官方导航说明](https://playwright.dev/python/docs/navigations)指出：load 前的客户端跳转由 goto 等待目标 load；load 后仍可能有动态活动，不能视作所有页面活动已结束。规则开始之后再跳转、删除自身状态、网络拒绝等仍 fatal，不重跑规则来恢复状态。新增纯策略／真实 render 调用的 mock 测试核对两条原有调用、相同超时与导航失败不执行规则；mock 不替代真实浏览器。HTML 注入路径原有“先 goto 后 setContent”也未在此改变，不能宣称完全等同旧参考的全部分支。

不得加入额外 goto、源脚本重执行、POST 重发、空结果成功、任意静默错误恢复或扩大预算／私网放行。源脚本执行后再发起导航仍须保持失败，不能用此修复承诺跨文档继续执行任意规则。

修复后的本地全量 Python 实际 197 项：196 通过、1 个 Windows 环境跳过、0 失败／错误，34.369 秒；新增 4 项包含生成 render 调用的 mock，不宣称真实浏览器通过。发布链结构／导入／manifest 的 Node 57 项实际全通过、无跳过。JDK 11 离线 `compileTestKotlin` 成功，4 分 25 秒；保持 2 个活动处理器／512 MiB／单 worker，未在本机启动真实引擎或原 JAR。58 份既有用户报告逐文件 SHA-256 与工作前快照一致，未读取正文或暂存进提交。

当前修改尚未完成自身真实 18 项／包内 worker／新镜像实站验收。原 JAR／旧引擎同输入、真实认证、长测／生产亦未由此完成。原失败与诊断证据继续保留。修复回滚只恢复 worker 的两处 DOMContentLoaded 与 source 读取位置，生成回归应保留暴露兼容性缺口；不要撤去门禁来把旧实站失败称为解决。未发版／部署／改用户数据。
