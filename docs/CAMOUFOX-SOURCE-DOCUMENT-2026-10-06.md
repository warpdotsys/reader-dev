# 源状态丢失时的有界文档引用观察

## 为什么继续诊断

`01459443` 的正常托管门禁及双架构导入已通过，但自己的匿名起点专项仍为一次详情 7.19 秒、HTTP 200／`isSuccess=false`／data null。原有限标签为 `sourceStateMissing` 且主框架导航事件为 true，没有页面快照。[原失败证据](evidence/public-metadata-load-snapshot-fix-hosted-2026-10-06.json)。同文档 hash／history 也可能产生主框架导航，不能仅凭这个事件认定新文档、验证码或唯一根因。

## 本增量的观察与界限

源规则仍只执行一次。同一次 START 在执行规则前捕获 document，返回一个 null-prototype 的不透明 holder；Python 使用 `evaluate_handle` 接收引用，不序列化 document、HTML、Cookie、结果或错误，不增加第二个页面全局标记。null-prototype 避免规则通过 Object.prototype.then 把返回 holder 变成待完成 thenable；原随机状态、Promise 原生 adoption、业务结果格式及读取方式不变。

[Playwright 的公开 JSHandle API](https://playwright.dev/python/docs/api/class-jshandle)支持不透明对象引用、传作 evaluate 参数及 dispose；导航／上下文销毁可能使引用不可用。这是 API 依据，不代替固定引擎的实际核验。

只有四种既有 SourceScriptStateLost 路径额外做一次只读相等比较，先后检查原单调 deadline 和网络拒绝检查。只附以下三种固定标签：

- `sameDocument`：成功读到严格 true，当前 document 与捕获引用相同；不证明谁删除了状态或实站已恢复。
- `differentDocument`：成功读到严格 false；不据此恢复／重放规则。VM 替换全局仅为语言夹具，不冒充真实浏览器导航。
- `unavailable`：引用跨上下文／已处置、读取失败、非布尔、预算过期或网络拒绝；不分类或输出异常文本，不证明新文档、原因或验证码。

成功、拒绝、超时和一般 driver 错误不增加文档比较；holder 在所有退出路径尝试 dispose，失败不替换原异常。没有 goto、POST、规则重执行、页面修改、预算重设或策略放宽。诊断 evaluate 与已有 evaluate／clear 一样由原父进程 watchdog 兜底；不承诺能在阻塞浏览器事件循环中读到标签，也不扩大 watchdog。

原 NDJSON `{"error":"SourceScriptStateLost"}`、公开 ReturnData 和 fatal 行为保持不变。stderr 新增可选 `sourceDocumentObservation`；采集器精确接受原三字段、历史四字段与新增五字段白名单，未知值／字段、重复键、URL、原日志、正文和凭据均拒绝。单行／总输入／记录数上限和 exclusive-create 不变。

## 先红后绿的本地验证

旧 worker 上新增四个受控 Python 测试实际 3 Error＋1 failure；缺失标签及未处置 holder 被明确暴露。加入实现后相关 64 项纯测试通过；再加入到期／拒绝不读、未知内部标签不回显、采集器可选值／重复字段拒绝及新小报告守卫，全量 Python 216 项中 215 实际通过、1 个 Windows 环境跳过，0 失败／错误，29.189 秒。它们包含 mock 时钟／IPC，不是真实浏览器。

worker 原样 JS 的 Node 夹具从 26 增至 29 项，核对执行前引用、无额外全局标记、恶意原型 then；经 Python wrapper 实际执行、无失败／跳过。发布结构／导入／manifest 的 Node 57 项实际通过，无失败／跳过。

本机 JDK 11 离线 `processResources` 实际成功，2 分 53 秒，2 活动处理器／512 MiB／单 worker；源 worker 与复制资源 SHA-256 同为 `2bc9fe2f...`。这是资源任务，不是新增 JVM 编译、JAR 启动或真实引擎验收；未启动原 JAR 或用户 Reader。新增报告 schema 严格整数校验后独立 5 项再次通过。58 份既有用户报告逐文件散列未变，未暂存这些数据。

## 此增量必须取得自己的真实引擎结果

新增 `verify-source-document-observation.py` 仅使用一个自有 loopback 生成夹具，页面 HTTP 请求白名单限两个路径，拒绝其他页面请求，禁下载／service worker／GeoIP／WebRTC、预取及附加 UBO。输出只有三项具名结果、严格文档标签／导航布尔／单次执行计数、是否通过和页面网络拒绝数；不接受账号或实站 URL，不保存 HTML／Cookie／错误／正文。不是内核全流量网络隔离或 Reader 代理安全测试的替代。

两个删除状态夹具实际调用产品 `evaluate_source_script`，分别无导航及 hash 同文档导航；必须保持 SourceScriptStateLost、只执行一次，并在同一页面完成后续独立规则。第三项是在生成夹具显式导航后比较原 holder，必须不可用；这是文档引用 API 的实际核验，不宣称跨文档状态恢复或端到端 Reader 三方一致。

Native 共享 JAR 和完整镜像工作流均在原 18 项门禁之后显式运行该真实 helper，60 秒外层限额，失败阻止后续导出／构建；Native 还先比对 JAR 内 worker 与源字节。分别保留小型 JSON 包括失败，由精确三项守卫拒绝缺项、重复、伪计数、失败及类型伪装。旧 18 项和本提交以前的绿灯不证明新增 helper，仍需此提交自己的真实结果。

本增量尚未证明起点恢复、真实认证、原 JAR／历史引擎同输入、本候选长测或生产。新的实站请求只应消费本增量自身已过正常门禁的镜像，保留已有失败，不导出用户浏览器会话或复用已清除凭据。

## 回滚

撤回 START holder／evaluate_handle、可选比较／dispose 和第五字段，恢复原 evaluate；撤回真实 helper 与对应工作流步骤。保留原实站失败和纯回归证据。业务规则、存储、账号、网络配置没有迁移或数据回滚操作。
