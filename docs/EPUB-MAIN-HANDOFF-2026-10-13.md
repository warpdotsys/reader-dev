# 主线程交接同轮记录：首次结果已处理，刷新末尾仍无回执

本轮继续完整目标，不改Java/Kotlin业务、原期限、资源限额或生产。取到了实际首次Worker结果解析和文档赋值，但没有拿到完整刷新/几何结果；不标为修复或正式验收。

## 诊断补丁及可回滚边界

准确源JAR仍为e31托管 `7d757415…24b811`。新独立临时JAR `2ceed483a92bc18184a98820a68bfce35084ac8ac3f407c60274dbbff91f1ab9`、285,651,286B / 1,568项，只改变以下两个静态文件内容，其余后端、依赖、MANIFEST及资源逐项相同：

- `epubArchive.worker-Ch6g2lIA.js`：沿用前轮7处固定console时间插入，原Worker `59ebfb6a…64ed9f1`、诊断版 `7334f2a4…1a525b`，完整1,673资源/49,382,784B对照及原消息协议验证已保留。
- `ReaderView-C-j6wqdD.js`：原内容SHA `22f2ab44…939e72`、诊断内容SHA `87bc8666…d4ae5f7`；AST仅8处固定阶段调用，标记展开入口、ready/expanding/files回复、成功resolve调用、文档加载/赋值/finally。所有原表达式字节、15秒计时器和错误/取消分支保持不变；不记录消息正文、路径、凭据或错误文本。

实际展开函数用纯生成Worker doubles做8种情况/16次原版与诊断执行对照：成功、提前取消、运行后取消、重复ready、阶段混合、无效files、错误事件、超时；回复、传移调用形状、终止和原15000ms期限一致，插入日志实际触发。该fixture不包含真实DOM、文档赋值或实际零拷贝，不借为界面通过。

没有page.route，不改HTTP缓存；未覆盖原B26、只读备份、正式7d或普通构建JAR。生成脚本、补丁前后两资源、诊断JAR及SHA绑定保留于自建build目录。回滚是停诊断实例、继续使用未改7d；正式成果不能依赖这份临时热诊断。

## AI已实际观察到的首次原版806交接

完整原JAR生成账号 → 恢复登录/导入/注销撤token/新上下文登录/标签隔离 → 八阅读步骤，仍仅获准《黎明之剑》。UID/GID10001、零caps/no-new-privileges、仅loopback，合计2CPU/2GiB、high1.5GiB、256PID、零swap，Xms128/Xmx768、8/8池、60秒下载/15秒Worker均未变。

| 同一上下文的阶段差 | 实测 |
| --- | --- |
| Worker input → decoded | 14,178ms |
| Worker decoded → postMessage调用返回 | 25ms |
| 主线程resolve调用 → loadEpubDoc返回 | 235ms |
| loadEpubDoc返回 → epubDoc赋值 | 198ms |
| epubDoc赋值 → finally进入 | 529ms |

主线程确实进入成功resolve调用，并记录到可读文档加载和赋值；不是仅“Worker已发送”的推断。该轮解码接近期限，不能借上一轮2–3秒称稳定性能。未收到`reply-files`入口单独日志，因此不猜其缺失原因；缺少末尾observation，也没有完整trace-error摘要，不声称所有日志校验零错。

展开入口在Worker构造和计时器注册之前，入口到resolve不能直接当作精确15秒计时器耗时。Worker与文档timeOrigin、内部performance.now及主机console接收属于不同上下文/通道，不将它们拼为精确消息延迟。上述首次场景也不证明硬刷新交接成功。

## 完整末尾仍未取得，不把未到达检查算成功

阶段协议完成文本805/806/806刷新、HTML806、原版806及原版805，最后进入原版805刷新。主机整体完成等待超时；原始诊断没有observation，停后独立检查已审定stdout协议也没有迟到完成结果，只看到同一首次Worker和主线程阶段。

因此没有刷新正文比较、最终iframe几何/可见性、aria-busy/fonts或明确刷新失败步骤；不将最后进入的阶段当根因，不计算八场景通过。170.962秒，内存峰1,630,396,416B、PID244、high289,878；硬限/OOM/PID限均0。这些计数不证明特定泄漏或唯一原因。

service已inactive/MainPID0、runc空后，逐项核验来源/绝对目标/SHA/UID1000/0444/单链接，只删除四份输入37,836,343B；无递归删除，所有来源及报告保留可恢复。原JAR、只读副本、唯一获准源EPUB和58用户报告哈希未变，未读取其他正文、个人会话或生产凭据。

[本轮有限回执](evidence/epub-main-handoff-incomplete-2026-10-13.json)，SHA `a74bac29f5c82a53219729a32f14a19deafc0a7a90baf6620a636a125aa5ee39`，绑定原始 `02a57a30…e078e7`、临时JAR、main AST来源、16次fixture和精确清理，不发布正文/私有书籍哈希。

## 新的框架代表性核查，不预先称存储是根因

当前自建框架将`/tmp`挂为512MiB tmpfs，Reader工作目录为`/tmp/actual-reader-generated`；生成账号、导入书籍、解析缓存等也被计入同一内存/零swap限制。采样最大shmem为186,589,184B；该汇总还包括浏览器共享内存、HOME/cache tmpfs等，**不能全部归给书架或从中推导某个组件独占量**。

仓库声明的生产配置 `deploy/reader-pro/compose.production.yaml` 则为 `./storage:/storage`、`./logs:/logs`磁盘绑定，`READER_APP_WORKDIR=/`；`/tmp`独立256MiB tmpfs。这里只确认配置文件，不是SSH核对了当前生产主机/磁盘或实际配置。未修改任何部署配置。

下一项应在相同2CPU/2GiB/high/期限、同一完整场景与获准单书内，核对磁盘绑定的独立生成存储对照；不挂真实storage，不提高预算，单独记录存储后端差异及回收，保留tmpfs压力失败结果。只有证明代表性差异后才解释它；不能直接把测试搬到更轻条件宣布原问题消失。

仍需要刷新主线程/iframe就绪的直接记录、严格正文一致性、最新整Native本机以及原件/现有远程WebView剩余三方协议/实站认证与长时稳定等完整门槛。goal保持active，不合并/tag/发版/生产部署。
