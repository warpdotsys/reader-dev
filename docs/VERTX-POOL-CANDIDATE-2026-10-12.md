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

完整镜像的 Docker/Compose 默认各8，可显式配置；不是全局缩小协程池，也不修改 event-loop、超时、压缩/展开大小、Worker15秒或资源限额。下面 P 轮已真正看到8/8和PID余量改善，但同书八场景仍失败，不能把这个候选称为完整修复。

本机 JDK11/Gradle6.1.1 离线已编译、7项新配置回归/零跳过通过；完整45套214项JVM测试、零失败错误、35项既有环境跳过。比较全部选项的测试使用这个版本实际支持的 `JsonObject.mapFrom`，没有伪造不存在的 `VertxOptions.toJson`。新4项Docker/Compose一致性负例及98项发布结构回归、结构检查通过。初次记录时自己的托管产物和单书重测待执行；后续结果另列如下，不覆盖原失败。

## 3d 候选自己的托管产物：已重建，不等于真实书阅读通过

源码 `3d3293ef8a164292d4fe5579bb73163a23b0ac32`／受测合并快照 `31273ebcb32a49e827a4b42b6fc8de23e33b02d0` 的四条托管流程已终态成功。Java两次clean重建得到同一JAR：285,675,118B、1571条目，SHA `cfe4dd0c9dc905f62ab794e41fb0cf58a52b5b007a54d75630bd3d37afc141e3`；下载后独立重算，并逐条与30e比较。新增一个Kotlin选项class，改变应用初始化相关六条目；依赖、前端、MANIFEST和浏览器worker均未变。不将这些初始化字节变化冒称原件所有行为完全等价。

[Native自己的运行](https://github.com/warpdotsys/reader-dev/actions/runs/38205074085) 六作业全部成功；32份publisher小JSON、两架构transfer及Cam/JVM XML已下载，经原消费器核验实际身份、默认界面、异步Cookie、Reader/Worker TLS及预算。748 Linux Python零跳过、214 JVM/35既有跳过、Cam24通过。[有限托管记录](evidence/vertx-pools-3d-hosted-native-2026-10-12.json) SHA `615fde61…93fd`。本机没有下载/重算两个大型镜像归档，不能据此称本机最新整镜像验收完成。

## P 轮同书真实 UI：8/8生效，raw仍失败

使用准确cfe4正式JAR、同一个授权EPUB和缓存401d的未变浏览器库；OCI环境明确设置两个池各8，其他预算、安全和八场景门槛不变。UID10001、无外网/仅lo、2CPU/2GiB/high1.5GiB/256PID/零swap，Java和浏览器合计；没有代理缓冲、Worker注入、HTML脚本或修改正文。

实际注册、导入预览/保存成功，前四场景与O的完整正文哈希都一致，806刷新前后也一致。Java峰47（event-loop3、两worker各8、协程4、scheduler1、JVM19、其他4），整轮峰PID234、PID事件0；预算子门槛接受。46.348秒，内存峰1,625,608,192B、soft-high72569，硬内存/OOM0。但raw806仍显示15秒解压超时，没有原版iframe；八场景仅完成四个，阅读总门槛拒绝。PID问题改善，Worker超时没有解决，内存回收压力仍在。

[有限失败记录](evidence/authorized-epub-cfe4-pool-pressure-2026-10-12.json) SHA `41e5cbda…aaf4`，绑定原回执 `183758c8…6a07`。P原回执继承了旧工具的 `javaOrBrowserThreadPolicyChanged=false` 自述，该标志不作为本轮线程策略证据；真实OCI配置和线程类别8/8是依据，原报告不篡改。未提交正文、逐章哈希、截图、凭据、线程原名或原日志。

P实例和采样已停、inventory0；只删除两份已校验且来源仍在的输入副本323,484,914B（约308MiB），没有递归删除。原件、授权源及58用户报告哈希再次全部通过。

## 下一项可读源码诊断：阶段确认不延长15秒

`epubArchive.worker.ts` 在模块加载后只发固定 `ready`，接收输入开始展开时只发 `expanding`，不附正文、文件名或大小。`epubWorker.ts` 按顺序接受纯阶段消息，拒绝未知、混合、重复或逆序；阶段不是成功，也不创建/重启计时器。超时只补充“等待Worker启动／等待接收文件／解压文件”，原15秒总限、终止、转移字节和大小上限不变。现有四项加五项阶段回归（其中三个用虚拟时钟在14998/14999/15000ms确认原期限）、全前端318项和类型检查通过；98结构回归也通过。

这是区分模块启动和实际展开的诊断，不是性能修复。自己的正式新JAR、生成UI及同书阶段结果仍待验，不能借3d或P作为新诊断的正式验收；本机依赖也不是新跑的精确 `npm ci`。生产不切换，文本/HTML已完成的四场景不能替代原版排版和后四场景。

## 当前托管参照与数据保护

前一维护提交c1be/74e自己的四条GitHub托管流程、Native六作业均终态成功。下载32份publisher JSON、两架构transfer和JUnit XML后，按原消费器重新验证身份、默认界面资源、异步Cookie、Reader/Worker TLS、预算；748 Linux Python零跳过、207 JVM/35既有跳过、Cam24通过。[自己的有限托管记录](evidence/thread-groups-c1be-hosted-native-2026-10-12.json) SHA `8a25ded7…267b`。没有本机下载/重算大型镜像归档，不把这份小报告消费当成本机最新整镜像、O轮单书或新线程配置的验收。

M/N/O的自有实例全部停止、PID0/delete，inventory0。只删除六份本次只读JAR/EPUB输入副本，共970,448,490B（约925MiB），可从保留来源重建；未递归删除目录。原JAR/只读备份/授权书源及58份用户报告哈希不变，其他本地正文未读。

## 已知限制与回退

当前正式30e仍有真实单书PID触限及raw Worker超时。候选8/8已观察到降低线程峰值、P轮不触PID限，但raw仍超时；并发延迟与完整原版排版兼容尚未验证。不抹去O/H/I/K/P及之前其他失败。真实认证、原件HTTPS/WS/WSS三方、OPDS等既有缺口也不由本次线程配置或阶段诊断关闭。

回退候选时，在独立干净工作树做定向源码回退，保留回归及原失败。使用包含该选项的新镜像时，可将两个变量显式设为20恢复原池大小（不保证能通过旧预算）；独立JAR不配置变量仍为20/20。不得 reset 用户工作区、提高预算或延长超时来伪造通过。本轮不合并、不发版、不推registry、不部署，goal保持active。

阶段诊断可在独立干净工作树定向回退上述Worker三个源/测试文件，再由同一托管流程重建；不回滚用户storage或导入数据，不改已经保留的O/P失败证据。
