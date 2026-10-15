# 准确原件的生成元数据三方验收

日期：2026-10-11。仅使用生成账号、书源、页面及正文；书名标签不代表读取过真实《黎明之剑》。原件、只读备份、准确恢复 JAR 和 58 份用户工作区文件哈希未变。不发版、不推 registry、不部署生产。

## 已成功执行并独立核验

本轮不是拼接单侧成功：同一隔离网络、同一生成目标及同一次比较进程实际执行以下三侧，每侧 5 次搜索和 1 次 getBookInfo，共 15 次搜索、3 次书籍信息请求。

| 侧 | 实际 JAR | 引擎 |
| --- | --- | --- |
| 原件 | b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c；72,913,887 B | 固定历史 WebView |
| 恢复版 | 9b9d0c9808968c0390022156a5fcbfcc03f595ed228f124afbb06e830cd7dd84；285,672,062 B | 同一历史 WebView |
| 内置引擎 | 同一准确 9b9d JAR；worker d8b2674c5f28cf31d9003479c7e5035c9d4ebefc0286c0d8a61c4d9672a565f5 | 隔离 Camoufox |

恢复 JAR 的下载来源仍是 a5cd1e74/2147efa3，字节与 **685aab2f 自己两次干净重建**独立核验结果一致。没有把下载来源改写成后续提交。历史镜像 manifest 为 sha256:88c250043bd33715ca372b749c1dca055c14e4550882caf968bd7af933d53573；本机只读 runtime 缓存仍为旧镜像 sha256:401d895c31a39634bf29654a7fbb29527272d287c8c1219b12794bbe1970bc44。此轮不是本机最新完整镜像验收。

- 每侧基础搜索的 HTTP 200、isSuccess=true、errorMsg=""、完整 ReturnData/data，以及 GET/POST 原始请求字段均通过；三侧完整基础响应相等。
- Cookie 导入、清除和脚本请求实际执行。原版 remote 行为和 Camoufox 的 Cookie 传递形式分别按原有严格契约验证，不把模型差异改写成相同。
- 普通表单 POST 的实际目标 Content-Type 精确为 application/x-www-form-urlencoded; charset=UTF-8。
- 延迟 DOM 的书名、作者、封面、全部非时间 JSON 字段、字段类型、默认值、诊断标志和规则保存读回通过。
- latestChapterTime、lastCheckTime、durChapterTime 严格各在自己的实际 API 请求窗口内。三个时间值及完整原始响应全部保留；**完整原始元数据 ReturnData 字面相等仍为 false**，只接受完整非时间 JSON 相等及各自新鲜时间窗口，不是删掉差异后声称字面一致。
- 严格比较器及发布守卫未放宽，没有业务请求重试、超时延长、TLS 例外或空实现。

[完整生成响应和独立验收回执](evidence/metadata-three-way-9b9d-reclaim-2026-10-11.json)，SHA-256 6e28297deb59d5b185d269357a14df4a8f92926e603ae5d4b7f0c9cacc7a83ce。原始外层报告 SHA-256 be547b914994222e10c06ad189905eb33e0c3bbbc899e5fe752d6e528c4b7edc。

## 已验证的测试交接改动；超时根因仍未证明

上一轮三方在 Camoufox 首搜索超时，保留在[原失败报告](HEADER-CONTRACT-DIAGNOSTIC-2026-10-11.md)。其原始报告 bf5af82a6b0d58d076a34cedaa56c0c6b7ece4c5ab24395c5cd63e15f55f5541 未被改写。之后恢复版单独运行完成 5 次搜索/1 次元数据，仍触发 memory.high，因此 high 事件本身不能证明超时原因：[单侧原始范围回执](evidence/metadata-standalone-9b9d-2026-10-11.json)。

新测试交接只增加一次对**本轮已停止、PID=0、cgroup 为空的历史实例叶节点**的缓存回收。使用内核 [memory.reclaim 定向接口](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory)，不写全局 drop_caches，不删除磁盘文件，不提高原资源限额，不重置累计 peak/events。护栏拒绝其它实例、宽路径、符号链接、活跃进程、子节点、缺失计数、限额变化；打开控制文件后再次验证。

本轮实测：

| 量 | 回收前 | 回收后 |
| --- | ---: | ---: |
| 停止实例 PID | 0 | 0 |
| 停止实例 memory.current | 545,071,104 B | 4,542,464 B |
| 停止实例 file 缓存计数 | 540,184,576 B | 0 B |
| 父组 memory.current | 1,119,764,480 B | 578,433,024 B |

一次命令请求 545,071,104 B；内核返回 EAGAIN（部分回收），记录为 kernelPartialReclaimObserved=true，**没有伪称完整请求成功**。实际测得叶节点计数减少 540,528,640 B；只在独立确认 PID 仍为 0、残余计数小于 64 MiB 后允许第三侧开始。

改动只涉及测试交接；Reader 生产源码和浏览器二进制未改。第三侧通过说明本次有限契约已完成，但没有重复随机化 A/B、冷/热缓存控制或充分根因证据，earlierTimeoutRootCauseProven 仍为 false，同源 Authorization 丢失也不能据此声称修复。

回执内执行脚本SHA对应[原样执行源码](evidence/reclaim-stopped-browser-cgroup-executed-2026-10-11.py)，2be31c58c2df28cf55af7166b2fd72e25e85a6573c719c168d711646206eae72。维护脚本仅清理文件末尾空行，除此以外文本完全相同；保留原执行字节，不回填报告哈希。

## 资源、清理和验证范围

UID/GID 10001、附加组为空、所有 capability 为 0、no-new-privileges、只读输入/root、独立 PID/mount/IPC/UTS、仅共享自有 user/net、网络仅 lo。父组 aggregate 2 CPU / 2 GiB / 256 PID / memory.high 1.5 GiB / swap 0。

45.947 秒；累计内存峰值 1,611,583,488 B、PID 峰值 242，high 3663；hard max/OOM/PID 上限事件 0、swap 0。24 个历史 PID-1 接管的僵尸由现有精确 reaper 回收。最终两个自有 runtime 均独立观察 stopped/PID 0 并删除，剩余 0；不是长期容量或无泄漏证明。

本轮结束后仅删除两个已核对精确绝对路径、所属 UID、只读权限、单链接、长度和 SHA 的临时输入副本，共 358,585,949 B；原件、只读备份、CI 下载在删除前后均重新散列保留，副本可重建，不递归删目录。清理回执 SHA e692f43e0c3eba6aca28c329dc2b6bfada8f4b19b2d882058abe82fa25786550。

新增 18 个回收护栏测试使用假文件/模拟系统调用，不冒称真实 kernel 验收；本轮真实定向回收由上述实际报告证明。Windows 全 Python 622 项 / 29.331 秒 / 零失败 / 1 个既有 POSIX 平台跳过。新源码仍须自己的托管运行，不能借 685aab2f 的绿灯接受后续提交。

## 685aab2f 自己的四条托管已核验

源 685aab2f3e2b4146b514c70fb521f19f5e3ae717，实际测试快照 356c1f2b1fc5500b390a2b5d635e67f2977d1a3f。以下均为 GitHub 托管 runner，无 self-hosted：

- [Java 38107451361](https://github.com/warpdotsys/reader-dev/actions/runs/38107451361)：604 Linux Python/零跳过；201 JVM/35 既有环境跳过；9b9d 两次干净 JAR 字节一致。[独立回执](evidence/header-diagnostic-java-685aab2f-2026-10-11.json)。
- [Vue 38107451359](https://github.com/warpdotsys/reader-dev/actions/runs/38107451359)：301 前端、25 实际生成 UI/零跳过，类型与构建成功。[独立回执](evidence/header-diagnostic-ui-685aab2f-2026-10-11.json)。
- [Full 38107451513](https://github.com/warpdotsys/reader-dev/actions/runs/38107451513)：Camoufox 24/零跳过、完整镜像默认生成 UI、实际 Reader HTTPS 等门禁通过；内存峰值 851,841,024 B/PID 199。[独立回执](evidence/header-diagnostic-full-685aab2f-2026-10-11.json)。
- [Native 38107451446](https://github.com/warpdotsys/reader-dev/actions/runs/38107451446)：六实际作业成功；AMD64/ARM64 构建与重新导入、32 个 publisher JSON 逐项验证，同一 9b9d JAR，四套真实 worker/Reader HTTPS 报告接受；资源峰值最大 980,799,488 B/PID 205。[独立回执](evidence/header-diagnostic-native-685aab2f-2026-10-11.json)。

本次 Full 的同源头契约未复现原 a5cd 失败。685 只增加诊断和记录回归，生产 JAR 没变，不把“这次通过”改写为“根因已修复”。Native 的镜像/archive 准确身份在回执内；未在本机重新散列大型 archive，也没有 registry 推送。

## 尚未验证与回退

真实书源认证/授权书籍旅程、准确原件完整 HTTPS 三方、WS/WSS、干净环境的一键本机 rootless 三方入口、最新完整镜像本机运行、OPDS、正式发版和生产部署仍未完成。上述生成元数据有限接受不替代这些目标。

若撤回本次交接优化，仅在没有活跃测试时移除其调用/单独 revert 回收护栏候选；原来的资源预算、比较器和失败记录保留，不 reset 用户工作树。缓存回收没有删磁盘文件或修改服务配置，不需要恢复浏览器二进制。临时测试输入可从保留的准确 JAR 重新复制，不能覆盖原件或历史报告。
