# EPUB 输出缓冲区与 JVM 核数：窄修复及有限验证

更新日期：2026-10-12。本文区分上一提交的托管构建、本次源码修正、生成样本观察和仍未验收的实际 Reader 压力。没有读取其他本地正文、真实凭据或个人浏览器会话，没有生产改动。

## 上一提交自己的托管证据

`eda9a0bab0015f65b5f1a089d177d8b03669b641` 的测试合并快照为 `a4544c9061bb077ff43dad96b1b8131312a91786`。四项 GitHub 托管检查及六个 Native 作业已终态成功；根代理重新执行冻结守卫并精确比较各自回执，没有覆盖旧回执。

| 检查 | 范围 |
| --- | --- |
| [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/38184012750) | Python725，JVM207含35既有环境跳过；双clean完整JAR字节一致 |
| [Vue](https://github.com/warpdotsys/reader-dev/actions/runs/38184012770) | 前端308，实际生成界面旅程25，零跳过 |
| [完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/38184012794) | Camoufox24；真实默认UI一个旅程含TXT及生成EPUB三模式、翻章、刷新、退出，12.85秒；四张登录/书架/TXT阅读/退出截图经实际查看、PNG解码及哈希核查，中文可读 |
| [Native](https://github.com/warpdotsys/reader-dev/actions/runs/38184012755) | amd64/arm64构建、重载和发布导入六作业；32份publisher有限文件独立核验，没有registry发布 |

本次下载的共享JAR为 `0675ec6dd851f737804e0f6d306b265f0b9e9b5df45fb62f79c9ad018aa2febe`，285,672,106B / 1570条目，重新散列与双clean一致；不是原始b26 JAR。没有下载或本机重算大型镜像归档。完整镜像合成子集峰值856,850,432B/PID195，publisher资源子集最大891,809,792B/PID197；不能当作全部TLS案例或客户端与服务端合计峰值。

镜像UI验收客户端在服务端预算之外，普通预算仍允许1GiB swap；不能取代旧H/I/K和本机Z的合计2GiB/256PID/零swap压力验证。四张截图不包含EPUB模式画面；模式内容与翻章由同一真实JUnit旅程的断言证明。该镜像UI旅程只在amd64执行，不能宣称arm64 EPUB界面也已实际测试。这些旧绿灯不覆盖下述新源文件。

## 实测 JVM 可见 CPU，不据配额改线程池

AA使用缓存runtime `401d895c…bc44`，不是上述最新整镜像。仅编译并运行只读Java11探针，不加载Reader、书籍、浏览器或凭据，也不挂载Reader JAR。UID/GID10001、无附加组、零capabilities、no-new-privileges、六namespace独立、仅lo和只读root/class均核验；CPU配额2、亲和性0/1、memory.max2GiB/high1.5GiB、256PID、swap0不改。

| 指标 | 默认JVM | 显式 `ActiveProcessorCount=2` |
| --- | --- | --- |
| `Runtime.availableProcessors()` | 2 | 2 |
| ParallelGC / ConcGC / CICompiler | 2 / 1 / 2 | 同左 |
| Java线程 / 最大堆 | 5 / 805,306,368B | 同左 |
| `UseContainerSupport` / override | true / -1 | true / 2 |

因此没有取得这一环境下核数误识别的证据，不把旧应用约70线程推定成“JVM看见32核”，不添加固定生产核数或改Vert.x线程池。它仍只是独立探针，不是Reader实际线程分类或PID压力定因。单元3.484秒/CPU594ms，父级峰值100,343,808B/PID19，soft-high/hard/OOM/PID事件/swap均0；唯一自有容器stopped/PID0/delete，inventory0，另外确认inactive/dead/MainPID0。

原始有限回执 `7c17c71e…eb66` / 6139B，绑定五个执行源码/class SHA和冻结namespace模板；没有反写Z或旧压力轮次。

## 已从维护源码修正：每资源一个输出目标

原 `expandEpubArchive` 先保留所有ondata分块，再在EOF分配完整资源并复制分块。新实现通过原中央目录预算及本地/中央一致性检查后，为非目录资源分配一个准确大小目标，每个分块先执行原实际输出与累计量检查，再写入该目标；EOF仍必须实际大小等于声明才能返回。目录只核验输出计数，不物化为文件；零字节文件仍保留。

**没有减少承诺的总复制字节或取消fflate内部复制**：仍需将各个解码分块复制进独占目标，只消除应用层“保留整资源分块集合并在EOF再建第二份完整目标”的同时强引用。预分配发生更早，对低报/高报/截断/畸形档案依然拒绝，不能返回补零或截断的伪成功。fflate在ondata前的临时扩容、流下载分块累积及全书物化仍存在，不能称严格常量内存或容量问题已修复。

压缩64MiB、展开256MiB、单资源32MiB/文本4MiB、8192条目、4KiB输入切片、15秒Worker及60秒下载期限全部不改；没有新增Worker、放宽超时、修改用户命名空间、后端或存储格式，也没有字节码热补丁。

新增3项真实fflate回归：多块stored/deflate和多资源逐字节/独占缓冲、未知本地大小的数据描述符、高报大小拒绝及空文件/目录。相关archive/loader/Worker共37项通过；本机直接 `node --test` 全前端311项通过，TypeScript检查通过。当前PATH没有npm，失败的npm尝试没有执行测试；实际采用包内既有的同一 `node --test` 命令。本机依赖目录不是新的 `npm ci`，不能冒称锁定干净重建。

## 生成资源三轮对照：内容一致，不保证均匀省内存

仅自行生成一份含两项32MiB二进制资源及生成中文章节/metadata的ZIP/EPUB输入。压缩84,200B、展开67,535,729B、6条目，SHA `082a8435…5558`；这是高压缩比解码样本，不是36MiB真实单书下载、1611目录、多模式UI或完整压力场景。旧源码git blob `766fa7b5…2160`，候选工作区源SHA `6431c8c4…a9f3`；每轮都用独立新Node进程、真实fflate0.8.3及同一字节输入，验证每项完整SHA及独占准确大小目标。

使用缓存Playwright SDK已有的Linux Node24.18.1，不安装新系统包。实际解码函数与fflate push只作委托式采样，未替换输出；采样和构造器观察有开销，不覆盖fflate所有瞬时分配。每轮应用目标分配数都为6、分配字节都为67,535,729，不能说总分配量下降。

| 轮次 | 解码ms 旧→新 | OS进程峰值B 旧→新 | ArrayBuffer采样峰值B 旧→新 |
| --- | --- | --- | --- |
| 0 | 271.860 → 244.355 | 279,552,000 → 252,493,824 | 166,392,810 → 149,479,712 |
| 1 | 217.674 → 274.802 | 265,814,016 → 261,255,168 | 143,054,418 → 149,479,999 |
| 2 | 240.024 → 229.859 | 254,349,312 → 254,201,856 | 135,332,044 → 148,197,742 |

六次完整输出摘要一致。OS峰值本样本三次略低，但两次ArrayBuffer采样更高、一轮更慢；GC与采样变化没有定因，不能宣传“固定降低32MiB”“总内存单调下降”或通用性能保证。单元7.089秒/CPU5.230秒，合计父级峰值461,430,784B/PID14，soft-high/hard/OOM/PID事件/swap均0；已确认inactive/dead/MainPID0。没有启动Reader/浏览器，也没有真实书输入。

原始benchmark回执 `bfe6bdb9…0f9d` / 5520B含候选源SHA及旧git blob，但没有benchmark主脚本/启动脚本执行SHA；现场文件哈希仅是工具调用顺序与未后改的补充，不升级成原始回执执行绑定。原始文件、脚本及生成输入全部保留，不通过反写弥补这一缺口。

[CPU及对照有限公开摘要](evidence/epub-copy-jvm-cpu-2026-10-12.json)由原回执逐字段投影、重新核验并精确比较，SHA `d0b6fc33…8550`，无正文、URL、Cookie、用户名或异常原文。原JAR/只读备份、授权单书及58份用户报告的哈希保护仍通过。

## 尚未验证与回退

本次输出缓冲区修正自己的托管构建、准确新JAR与实际镜像UI仍须另验，不能借eda9四绿。fflate瞬时工作集、真实单书合计压力、应用/浏览器线程分类、完整八场景及原件/远程/current三方、实站认证、同章anchor、OPDS服务和生产切换仍未闭合；Z的一条4xx也仍未逐操作定位。

回退时在独立干净工作树只恢复旧chunks收集和EOF合并，形成定向回退提交；保留回归、诊断、失败记录和输入保护，不reset用户工作区。回退会重新引入该应用层同时持有两份资源的路径，不会修复其他未闭合项。本轮不合并、tag、发版、推registry或部署，完整goal保持active。
