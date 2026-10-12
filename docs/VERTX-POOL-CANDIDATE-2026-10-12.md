# 真实单书 PID 触限与 Vert.x 线程池配置候选

记录日期：2026-10-12。原失败保持失败；本文的源码候选尚未通过新正式产物的单书重测，不表示已解决全部压力或可以部署。

## 已实际复现，不用 CI 绿灯覆盖

O 轮使用准确恢复 JAR `30e8cafcea6b07781aae8039c5ae0230340c189884f61eba445bab72e5b440af`、用户唯一授权的《黎明之剑》EPUB（37,809,796B、SHA `bd8c6819…ced31`）。只读输入、UID/GID10001、零 caps/no-new-privileges、私有仅 lo 网络与命名空间均由宿主验证。浏览器及 Java 合计仍限 2CPU/2GiB、high1.5GiB、256PID、配置及实际 swap0；不注入 Worker/HTML 脚本，不经小样本缓冲代理，也不读取其他本地书。

界面实际注册、导入预览和保存成功，text805、text806、刷新806、HTML806 四场景完成。刷新前后同章完整正文哈希相同，文本/HTML 无替换字符或 script；正文只在私有进程内比较。切原版排版时界面出现 15 秒 Worker 超时，原版 iframe 未形成，后四场景未完成。

整轮 93.149 秒，服务 100.270 秒/CPU128.702 秒；峰 1,624,145,920B/PID256，PID触限1、soft-high117515，硬内存/OOM0。完整八场景与合计预算均拒绝。内层真实 UI 首个错误为 timeout，外层验收拒绝为 validation，分别记录，不能把后者当业务错误唯一原因。

83 次有限采样中，首次 PID 事件发生在 `uiRefresh806Text`，早于切 raw。最大 Java 样本为 71 线程，主要类别如下：

| 类别 | 实际线程数 |
| --- | ---: |
| Vert.x event-loop | 3 |
| Vert.x internal blocking worker | 20 |
| Vert.x普通 worker | 20 |
| 协程调度 | 5 |
| JVM运行时 | 18 |
| 调度与未知 | 5 |

导入前 Java 为29；导入预览 internal worker增长到17，保存时普通 worker20，阅读后两池各20。浏览器刷新同时有瞬时任务增长。这个结果排除了“这里的增长主要是64个协程线程”的猜测，但还不证明两个池是 Worker 超时的唯一原因；内存高水位、浏览器瞬时任务和其他开销仍有风险。

[有限失败摘要](evidence/authorized-epub-30e8-thread-pressure-2026-10-12.json) SHA `829739d3…4eb7`，绑定原始回执 `36ea2766…5e16`。不提交正文、正文截图、账号、Cookie、线程原名或原日志。M/N 的路径及模块导出工具错误保留，未作为产品压力结论；O才是真正完成导入并进入阅读的本轮失败。

## 已回到可读 Kotlin 源码的候选

`ReaderVertxOptions.kt` 显式读取两个环境变量：

- `READER_VERTX_WORKER_POOL_SIZE`
- `READER_VERTX_INTERNAL_BLOCKING_POOL_SIZE`

未设置时直接保留 Vert.x3.8.5 的20/20及全部其他默认选项。显式值只接受1–128的整数，非法值启动时拒绝且不回显原值。`ReaderApplication` 在首次创建同一个共享 Vert.x 实例时使用该选项；不另起实例、不改变路由、用户命名空间或数据格式。

完整镜像的 Docker/Compose 默认各8，可显式配置；不是全局缩小协程池，也不修改 event-loop、超时、压缩/展开大小、Worker15秒或资源限额。目标是减少本轮两个20池的任务占用，尚需受测新 JAR 中真正看到8/8，并用同一真实书完成八场景，才能接受实际改善。

本机 JDK11/Gradle6.1.1 离线已编译、7项新配置回归/零跳过通过；完整45套214项JVM测试、零失败错误、35项既有环境跳过。比较全部选项的测试使用这个版本实际支持的 `JsonObject.mapFrom`，没有伪造不存在的 `VertxOptions.toJson`。新4项Docker/Compose一致性负例及98项发布结构回归、结构检查通过。新提交自己的托管产物、请求/界面兼容及单书重测仍待执行；本机编译不冒称正式镜像已成功。

## 当前托管参照与数据保护

前一维护提交c1be/74e自己的四条GitHub托管流程、Native六作业均终态成功。下载32份publisher JSON、两架构transfer和JUnit XML后，按原消费器重新验证身份、默认界面资源、异步Cookie、Reader/Worker TLS、预算；748 Linux Python零跳过、207 JVM/35既有跳过、Cam24通过。[自己的有限托管记录](evidence/thread-groups-c1be-hosted-native-2026-10-12.json) SHA `8a25ded7…267b`。没有本机下载/重算大型镜像归档，不把这份小报告消费当成本机最新整镜像、O轮单书或新线程配置的验收。

M/N/O的自有实例全部停止、PID0/delete，inventory0。只删除六份本次只读JAR/EPUB输入副本，共970,448,490B（约925MiB），可从保留来源重建；未递归删除目录。原JAR/只读备份/授权书源及58份用户报告哈希不变，其他本地正文未读。

## 已知限制与回退

当前正式30e仍有真实单书PID触限及raw Worker超时。候选8/8是否降低峰值、是否影响并发延迟、是否足以解决原版排版，尚未验证；不抹去O/H/I/K及之前其他失败。真实认证、原件HTTPS/WS/WSS三方、OPDS等既有缺口也不由本次线程配置关闭。

回退候选时，在独立干净工作树做定向源码回退，保留回归及原失败。使用包含该选项的新镜像时，可将两个变量显式设为20恢复原池大小（不保证能通过旧预算）；独立JAR不配置变量仍为20/20。不得 reset 用户工作区、提高预算或延长超时来伪造通过。本轮不合并、不发版、不推registry、不部署，goal保持active。
