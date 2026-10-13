# Worker完成与结果交接诊断：计时已取得，正式阅读仍未接受

后续同轮主线程成功resolve/文档赋值记录、仍未取得的刷新末尾及生成存储代表性核查见[主线程交接诊断](EPUB-MAIN-HANDOFF-2026-10-13.md)。本页原始AF/AG/AH结果保留。

继续Java/Kotlin主线。本轮不改产品算法、协议、预算、60秒下载/15秒Worker或生产；原件及正式7d JAR保留。取证不是修复交付，也不能借用已有托管绿灯接受临时产物。

## 已核对的诊断方式

从准确e31托管JAR `7d757415…24b811`提取实际Worker `59ebfb6a…64ed9f1`，用TypeScript AST只插入7处固定console计时：booted/input/decoded/posted/error/closing。仅记录kind、Worker timeOrigin及performance.now，不记录文件名、正文、输入、Cookie、token或异常文本；原解压表达式和postMessage协议逐字节保留。

两次新Node VM子进程验证所有1,673资源名称和49,382,784B输出完全一致、原三个消息回复不变、诊断五个成功阶段实际触发。每子进程15秒、256MiB old-space；VM不是实际浏览器、零拷贝或真实阅读接受证据。

AF先尝试页面级route覆层。实际看到同源Worker创建、请求与200响应，但覆层处理次数不存在、console阶段记录0；没有用这个失败的安装步骤推断解码时间。页面本身完成四个阅读场景，首次原版iframe可见性等待超时。初次生成器对出现两次的挂载文件名单误用单次替换守卫，实例创建前拒绝；修正为明确两处后才运行，不当作产品结果。

随后生成独立临时诊断JAR，SHA `3f87c233abc401941651a30d571ed01113567e3446754f1a0d285389480023e7`、285,651,045B / 1,568项；逐项核对仅 `BOOT-INF/classes/web-vue3/static/epubArchive.worker-Ch6g2lIA.js` 的内容变化，所有后端、依赖、MANIFEST和其他资源字节不变。Worker为 `7334f2a4…1a525b`。AG/AH不使用page.route、不改变HTTP缓存策略；这仍是静态资源临时热诊断，不能当作正式源码/JAR。

回滚只需停止自建实例，使用保留的未改7d JAR；诊断文件位于隔离build目录，未覆盖原始B26 JAR、只读备份、普通构建产物或生产。补丁内容、原Worker、AST生成器、正确性回执及临时JAR均保留。

## AG实际完整页面：解码完成，失败从超时转为刷新正文校验

仍用原JAR自行生成的隔离账号、真实恢复登录/导入/注销撤token/新上下文登录/会话隔离、唯一获准EPUB和完整八阅读步骤。UID/GID10001、零capabilities/no-new-privileges、仅loopback、合计2CPU/2GiB、high1.5GiB、256PID、零swap不变。

| 独立Worker | input到decoded | postMessage调用 | decoded/posted两条console的主机接收间隔 |
| --- | --- | --- | --- |
| 首次原版806 | 2,241ms | 27ms | 12,188ms |
| 原版805硬刷新 | 3,116ms | 65ms | 3,948ms |

两组booted/input/decoded/posted/closing均完整，console校验错误0。Worker内部时间与主机console接收是不同通道；上述接收间隔**不是Reader结果消息延迟**，postMessage返回也不证明主onmessage已处理。计时插入可能扰动调度，不声称无修改正式版的稳定性能、唯一根因或超时已修好。

文本805/806/806刷新、HTML806、原版806及原版805完成。硬刷新这轮iframe已出现、未显示Worker超时，但在原有严格断言 `refreshed_raw.bodySha256 == raw_first.bodySha256` 失败；返回文本805未执行。**六/八仍拒绝，不把新失败转成通过**。114.659秒，峰1,623,842,816B、PID247、high131,485；硬限/OOM/PID限及采样错误0。

这改变了下一行动：不能继续只猜解压循环；需要直接看主onmessage/结果处理、页面就绪及正文一致性。尚未确定是章节错误、字体/布局变化、瞬态采样或其他问题。

[AF/AG有限证据](evidence/epub-worker-handoff-2026-10-13.json)，SHA `aac0f3a3a85bfb0249aa9ac6a6bf6b2352eb17edfb36696125e6d432ac10963b`，绑定原始AF `4d6e73c2…0b2cad`、AG `8ba1833c…01fdb4`、诊断JAR、消息协议/全资源VM核验和清理。没有公布私有书籍哈希或正文。

## AH只增加比较诊断，比较步骤未到达

阅读源码检查发现初始化已有“合法?chapter最高优先级”，没有据此擅改保存进度。iframe自身会等待fonts.ready/3秒兜底和两次动画帧，并用aria-busy表示恢复中；既有测试只等待正文长度，没有读取这个就绪标志。这是后续要核对的事实，不是已证明的根因。

AH保留同一临时JAR和所有原判定，增加仅在私有内存中比较innerText、textContent、去空白文本及heading的哈希；输出前删除全部私有哈希，只准备输出相等标志、长度、章节查询数字、frameBusy/fontStatus。没有降低正文SHA、字数、段落、字符或沙箱门槛。

AH只完成文本805/806/806刷新及HTML806，首次原版iframe的可见性等待先超时；稍后失败快照iframe元素存在、未显示Worker超时，Worker解码6,247ms、发送调用54ms。**刷新比较根本未执行**，不能推断正文差异类别，也不能把稍后DOM存在当作它在原等待期内可见。105.531秒，峰1,621,303,296B、PID246、high100,808，硬限/OOM/PID限及console校验错误0。

[未到达比较的有限回执](evidence/epub-handoff-comparison-unreached-2026-10-13.json)，SHA `a22ecbde8a93cff206a11d1bfc5437983379bd84d45bb84e3d176d40955db861`，保留原始AH `2e660b10…ab987a`，不把计划或未执行比较记成结果。

## 保全与下一直接证据

AF/AG/AH全部终态，service inactive/MainPID0、runc空后，逐一验证明确绝对目录、UID1000/0444/单链接、大小、来源和SHA，只清各四份自建输入，共113,508,586B。所有来源、脚本、原始成功/失败回执、正式及诊断JAR保留，未递归删目录，可从保留来源恢复。原始JAR、只读备份、获准EPUB与58用户报告重复哈希一致，未复制其他正文、个人浏览器会话或生产凭据。

下一次应同时输出主onmessage进入/完成、文档赋值、iframe几何可见性、aria-busy/fonts就绪及原严格正文比较的有界状态，不再只依赖“expanding”文字或console通道推断。保留完整八业务场景和原限额；不提高内存/CPU、不延长Worker超时，不发版部署。

正式大书稳定阅读、最新整Native本机、原JAR/现有远程WebView三方剩余协议与实站认证、长时稳定和其他功能仍不满足整体目标；goal维持active。
