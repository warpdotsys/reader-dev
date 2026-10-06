# 源脚本状态丢失：有限诊断与不重放契约

## 已观测与本增量的边界

修正书源格式后的实际匿名专项 `37446333775` 在 4.375 秒返回 `SourceScriptStateLost`；同一镜像的无脚本专项得到无 body 文本、无元数据选择器的快照。两次 HTTP 书源读回均完整、预算与清理通过，但业务门禁仍红。[原始脱敏证据与限制](PUBLIC-METADATA-IMAGE-2026-10-06.md#修正书源后的同镜像独立配对复验)。

此前 worker 有四条不同检查都产生同一协议错误。只由类别不能认定导航、验证码、Promise 或唯一根因。本增量让下一次实际请求能区分发生在哪条检查；不改变既有错误、成功条件、导航／脚本／POST 次数或预算。尚未修复实站，也不关闭旧失败。

## 诊断标签

| 有限 kind | 对应实际检查 | 不可据此推断 |
| --- | --- | --- |
| `sourceStateMissing` | 读取结果为 null | 一定是页面跳转而非状态被删除 |
| `sourceStateTypeInvalid` | 读取结果不是对象 | 原页面内容或未知值是什么 |
| `sourceStateBodyInvalid` | done 的 body 不是字符串 | 原 body 数据是什么 |
| `sourceStateStatusInvalid` | 状态／键集合不符合固定协议 | 原状态文字或其他字段内容 |

只在此异常的固定 stderr 记录中增加 `mainFrameNavigationObserved` 严格布尔值。监听范围是本次规则等待、只看主框架；监听器 finally 移除。它只证明观察到事件，同文档导航也可能触发，不能当成文档替换或因果证明。使用 [Playwright 官方事件订阅／移除接口](https://playwright.dev/python/docs/events#addingremoving-event-listener)，但实际版本的运行正确性仍须自己的托管测试。

NDJSON 仍为 `{"error":"SourceScriptStateLost"}`，公开 JVM 包装／ReturnData 不变。标签不含原错误文字、URL、随机状态 key、Cookie、HTML、正文或结果值。未知／非布尔内部标记保持 unclassified，不拼入日志。

## 公开探针的安全采集

`collect-browser-failure.py` 仅处理本次自有容器的末尾 200 行流；普通 Reader 日志不保存或回显。单行最多 4 KiB、总输入 256 KiB、最多 8 条记录，超限标记 truncated，不冒充完整诊断。原三字段诊断和新四字段诊断必须精确匹配固定 phase／kind／errorClass／严格布尔；重复键、未知字段、未知值、非法 UTF-8 和续行注入全部拒绝。

只产生独立 `WORKER_DIAGNOSTICS.json`，使用 exclusive-create，存在则拒绝覆盖。原八份身份／资源／业务／清理报告仍保留。采集属于辅助观察，采集失败不能把原业务 exit 1 改为成功，容器仍 finally 移除。不会导出完整日志、站点 Cookie 或浏览器会话；未使用原 JAR、用户数据、认证或正文。

## 不重放与回归门禁

新增真实默认引擎契约 `generatedSourceStateDeletionFailsWithoutReplayingSideEffectsAndRecovers`：纯生成页面的源脚本先执行一次生成 POST，再明确删除自己页面的随机临时状态，返回未完成 Promise。要求得到原 `SourceScriptStateLost`，初始导航／原 POST／源脚本 POST 均恰好一次，随后独立新浏览器上下文实际恢复。它证明状态丢失可以不经过导航，不是实站故障复现或实站修复。

默认引擎门禁从 16 增至精确 17 个具名契约，旧 16 项 XML 不能充当此新增验收；全环境跳过、重复项、伪零计数和失败照常拒绝。本机没有最终 UID 10001 引擎环境，不以局部编译或 Python double 冒充真实 Camoufox。新的 17 项必须由 GitHub 托管工作流实际执行并消费自己的 XML。

本机 Python 192 项，191 实际／1 个 Windows 环境跳过，0 失败／错误；其中 worker 43、采集器 6，覆盖四原因、主／子框架区分、监听清理、原错误及 NDJSON 身份、日志拒绝和界限。Node 发布守卫 57 项无跳过／失败／错误，Bash 语法通过。JDK 11 离线 `compileTestKotlin` 实际成功（2 个活动处理器、512 MiB、单 Gradle worker），只是编译，不是执行新增真实引擎契约。它们不证明实站／认证／原件三方／长期或生产。

## 现有候选与新候选不要混写

`11c9595d` 的四个正常工作流已全部终态成功。完整镜像与原生共享 JAR 实际默认引擎 XML 各 16 项全部通过、0 跳过／失败／错误，SHA-256 分别 `ff08fcbe...`、`a5395750...`；按该提交自己的旧 16 项守卫实际核验，不使用新 17 项守卫追认。其实际托管书源／详情 JVM 9 项全部通过，XML SHA-256 `5d581d5a...`。

原生导入 `37446289416` 的 7,636 B 制品已独立下载：精确 20 JSON 文件集逐个重算散列，两架构原生／重新加载的四份异步及四份预算原守卫均实际通过，运行 JAR／版本／迁移身份匹配。共享 JAR `58e039c6...`、实际 merge revision `81a47855...`；预算配置允许 1 GiB swap，观测用量 0，不能写成配置零 swap。[自己的终态与小型制品证据](evidence/source-roundtrip-normal-hosted-2026-10-06.json)。这些旧候选结果不证明本增量，实站失败仍单独保留。

## 已知问题与回滚

实站仍失败；旧连续跳转通用 Error 仍未唯一归因；用户浏览器已有会话与匿名容器条件不同。不得增加脚本重执行、POST 重发、失败恢复为空值、任意 URL／Cookie 输入、私网放宽、root 回退或绕过验证码来获取绿灯。

回滚只撤掉 worker 标签／事件观察、专用过滤器／探针采集及第 17 个生成契约；旧失败和新诊断证据保留。没有状态恢复策略、超时放宽或用户存储修改需要回滚。若新标签仍未知或截断，必须如实记录，不能由旧 70 字符错误反推。
