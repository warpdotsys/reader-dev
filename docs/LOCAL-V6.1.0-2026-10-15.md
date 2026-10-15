# v6.1.0 本机实际测试（2026-10-15）

同日后续已建立可保留数据的本机完整正式镜像入口 `127.0.0.1:18963`，正常许可证激活、生成账号/EPUB/内置浏览器和停止/重启验收见[持久本机交付](LOCAL-PERSISTENT-V6.1.0-2026-10-15.md)。下文临时压力测试与 Windows 演示的原证据继续独立保留。

Windows 正式 JAR 的生成数据界面测试、准确 Linux 正式镜像的 850 章生成 EPUB 压力测试均通过。内置浏览器首次并发测试失败；后续同镜像 A/B 证明本机测试 PID 1 未回收孤儿进程导致限额耗尽，补齐回收后完整生成书源测试通过。修正的是测试环境，不是重新构建产品；生产 Compose 原本已有 `init: true`。未使用个人 Chrome、起点登录态或私人正文，没有修改产品或生产服务。

## 已验证：Windows 正式 JAR 与实际界面

直接运行正式字节 JAR，没有重编或替换前端。SHA-256 为 `b4954ac70229f4514146b8e077909874bfc5aeee779f71fde903622638ed1ee0`；MANIFEST 为 6.1.0、ReaderApplicationKt、Spring Boot 2.1.6.RELEASE。仅监听 127.0.0.1，独立生成账号和存储，未使用个人 Chrome、起点登录态或私人正文。

本机回执 `D:\Reader-Pro-Local-610-20261015-d\result.json`，SHA-256 `b70738392d758424b870df0df837c4160c2d6fa32580d5646b6c22de33b964f7`。12 项检查通过，页面异常 0，外部网络请求 0：

- 实际界面注册、登录、错误密码提示、退出；桌面和手机宽度中文截图已人工查看。
- 不记住登录时的刷新与新标签隔离；退出后真实接口拒绝旧令牌。
- 生成书源保存后实际界面展示、搜索、加入书架及两章阅读。
- 实际文件选择、EPUB 预览、保存与用户空间。
- 8 个小 EPUB 阅读场景：文本第 1 → 2 章 → 刷新 → HTML 第 2 章 → 原版排版第 2 → 1 章 → 刷新 → 返回文本第 1 章。比较完整可见正文，不截断为前缀或只核标题。

样本为程序生成的 3 章 EPUB（69,899 B，展开 259,994 B），不是 36 MiB 大书压力测试，不是真实书源认证，也不是 Linux 镜像浏览器验收。Windows JAR 本身不携带 Linux 浏览器运行时。

同日追加实际界面补查：本机演示中的第 2 章页面标题、正文标题和目录标题完全一致；点击目录第 1 章后，三者同步为第 1 章，刷新后仍为第 1 / 3 章。回执为 `D:\Reader-Pro-Local-610-20261015-d\title-refresh-check.json`，截图为 `screenshots/09-title-refresh-confirmed.jpg`。此项只补证 Windows 小生成书的可见标题，不追溯改写下文 Linux 850 章压力测试的标题覆盖缺口；原 `result.json` 未改动。

## 失败尝试保留

同日 a / b / c 隔离目录没有覆盖：a 在业务运行前因测试启动参数给出空 licenseServerUrl 失败；b 把容器入口生成的 `/assets/reader-release.json` 误当普通 JAR 必备文件，该不存在的路径返回 500；c 用 HTML `textContent` 把不显示的 `<title>` 一并计入正文，严格断言失败。d 修正测试假设，使用完整可见 `innerText`；没有修改产品、截短正文、放宽超时或隐藏旧失败。

## 本机演示

另启动的演示仍在 `http://127.0.0.1:18961/`，使用上述生成账号数据，内置浏览器已实际登录并观察到两本生成书。它不是自动测试留下的未清理子进程。来源服务同样仅监听本机 18962；独立停止脚本先核进程启动时间与可执行路径，只停止本次两个演示进程，不影响用户原 Reader。

## Linux 正式整镜像身份

锁定正式 AMD64 配置 `sha256:57403ef4e3d7412b049636985e4f3d2c950e3aaf94d917b21520c6c8309421d9` 和原 Native Actions 归档，没有用旧 f6 镜像或给旧镜像替换新 JAR 冒充。

已完成外 ZIP（2,174,950,743 B，SHA-256 `f16006ee7eeb3a5bccd0012d4b6f08fa5d2b3f5f47136f180232b141e56de0b3`）、内镜像归档（2,174,889,035 B，SHA-256 `70538a4f3e8b145d26ec39e8df183c347f4d470c5f07d06f73a7ccb0fd815d51`）和 14 层摘要核验，rootful 解包后再次核实正式 JAR。原 Docker config 字节保留；OCI 转换重新序列化 config 后，只在 JSON 语义完全相等时恢复原 config 字节并新建 wrapper，未修改文件系统层。

关键权限实测为：`/app` root:root / 0755、JAR root:root / 0644、`/home/reader` 10001:10001 / 0750、原入口 root:root / 0755。没有把全部文件的元数据声称为逐项独立验证。包内 Camoufox 0.5.6、Playwright 1.62.0、浏览器 152.0.4-beta.30 元数据匹配固定资产摘要；未把该元数据检查冒充重新下载浏览器 ZIP 核验。

准备回执为 `D:\Reader-Pro-Local-Image-610-20261015-a\prepare-result.json`。运行时使用准确镜像的完整根目录写时复制层、原入口、UID / GID 10001、空 capabilities 和 no-new-privileges；仅回环网络，无外网、无宿主浏览器会话。每轮结束后重新核验镜像配置及 JAR，原软件层未修改，容器库存清空且写时复制挂载已卸载。

## Linux 生成大 EPUB 压力测试通过

正式包内 Camoufox 实际驱动 Vue 3 界面完成测试，耗时 67.133 秒。样本为 850 章、38,626,835 B 的生成 EPUB，展开 93,511,402 B；不是实际藏书或原版 JAR 差分。本轮未放宽下载 60 秒、Worker 15 秒、界面观察 30 秒或总运行 220 秒的期限。

- 正确 / 错误密码、生成账号注册及重新登录、记住 / 不记住登录、刷新与新标签页隔离通过；退出后的旧令牌被真实接口拒绝。
- 实际界面完成 EPUB 预览与保存；导入弹窗关闭后重开为空，旧定时器不会误关新弹窗；重新登录仍可访问生成用户书架。
- 阅读场景为 `text805`、`text806`、`text806Refresh`、`html806`、`raw806`、`raw805`、`raw805Refresh`、`text805Return`。原版排版刷新前后完整可见正文、textContent、非空白正文及标题比较均相等，均为 9,486 个可见字符、64 段。各场景未观察到替换字符。文本模式转为纯文本段落，不保留 h1 / h2 / h3，因此只查询这些元素的辅助标题指标为 false；代码检查没有发现该指标所指示的章节错位。文本模式验证了章节索引、换章正文变化、刷新与回退一致性，但缺少独立的可见标题等于目录标题断言，不能声称所有模式都完成了该标题验收。
- 总预算 2 CPU / 2 GiB、禁用交换；包含 Reader、浏览器、测试程序与生成样本。总内存峰值 1,625,755,648 B（约 1.51 GiB），触发 1.5 GiB 软回收阈值 8,116 次；硬上限、OOM、OOM kill 与 PID 上限事件均为 0。它是整个测试组的内存计数，不是 Java 独占内存。

证据目录：`D:\Reader-Pro-Local-Image-610-20261015-a\runtime-evidence\reader-native-610-pressure-c0ed674a927e-61hixln7`。`runtime-report.json` 的 completed、资源检查、软件身份复核及清理检查均通过，`output__pressure-observation.json` 的 passed / readerEightScenesAccepted 均为 true。

## 本机并发失败的复现与环境修复

准确镜像在同样 2 CPU / 2 GiB 总预算、叶级 256 PID / 线程限额下，执行 `scripts/smoke-local-webview.py` 默认 4 请求并发阶段失败：`POST /reader3/searchBook` 返回 HTTP 200，但业务失败为 `Camoufox 渲染失败 (TargetClosedError)`。总组记录 PID 上限事件 5 次，内存峰值 1,435,271,168 B，OOM 为 0；因此不能仅凭 HTTP 200 判为成功。

初次失败时仅有错误与限额计数，不足以确定原因。随后使用同一正式镜像、同一份测试源码，固定 2 CPU / 2 GiB、256 PID / 线程叶限额、4 请求和原有期限，增加每 100 ms 的有界进程元数据采样；两侧唯一控制变量是测试 PID 1 是否回收已被领养的僵尸进程。直接 Popen 子进程仍由原调用者取得真实退出码，回收器不会抢走其 wait 结果。

| 对照结果 | 不回收的 A 组 | 启用回收的 B 组 |
| --- | --- | --- |
| 完整生成书源测试 | 失败，退出码 1 | 通过，退出码 0 |
| 采样 PID 峰值 | 256 = 212 活跃线程 + 44 僵尸 | 216 / 256 |
| PID 上限事件 | 4 | 0 |
| 回收数 / 结束时僵尸数 | 0 / 44 | 56 / 0 |
| 总内存峰值 | 1,235,300,352 B | 1,020,407,808 B |
| 内存硬上限 / OOM 事件 | 0 / 0 | 0 / 0 |

A 组的 44 个僵尸均为 PPID 1；B 组没有改产品字节、降低请求数或提高限额，4 个并发请求各返回一本正确的生成书。GET / POST 方法、请求体和请求头、实际脚本结果、两用户 Cookie 隔离、写入 / 重放 / 删除、异步脚本及延迟 DOM 元数据均通过。两侧采样无截断或错误，退出后容器、挂载及原软件身份复核通过。

因此，本机测试运行器未回收孤儿进程是此次 PID 额度耗尽的已验证原因；此前失败回执仍保留为失败，不改记成功。该结论不外推为所有 `TargetClosedError` 的原因，也不证明生产存在相同故障。正式 CI 使用 `--init`，生产 Compose 使用 `init: true`，均已具有回收器；这次 A/B 不是生产负载或长期稳定性验收。渲染器仍是单工作线程队列，4 个并发 HTTP 请求不等于 4 个同时运行的浏览器。

该测试只对生成回环书源启用了正式 CI 同样使用的 `READER_BROWSER_ALLOW_PRIVATE_NETWORKS=true`。开关仅作用于无外网隔离测试，未改生产默认值；之前未加开关的一轮被默认网络策略拒绝，失败证据同样保留。

证据目录：`D:\Reader-Pro-Local-Image-610-20261015-a\runtime-evidence\reader-native-610-smoke-98ef6b6c7a69-vk4zd0g5`。`runtime-report.json` 的 completed 与 aggregateResourceEventsAccepted 均为 false，`output__smoke-command.stderr.log` 保留第 374 行并发阶段异常，`output__smoke-reader.log` 保留浏览器错误。容器和写时复制挂载已正常清理。

后续 A/B 索引与摘要在 `D:\Reader-Pro-Local-Image-610-20261015-b\ab-result-index.json` 和 `diagnostic-summary.json`。A 组 `reader-native-610-smoke-67b504fd5c52-cm43a__b` 的进程观测 SHA-256 为 `e69d75ed671b0d862200c103027c6dfa2c937f916ba01f641072164431236125`；B 组 `reader-native-610-smoke-05484b5786fd-oxcd05gh` 为 `d933a059d551717bec30898fbec72c4a3295e3bd3e6acd0e8091400651628f4e`。两侧 namespace 诊断源码均为 `54d25483b2b13ea8ca8f2efe6bbd6c8fe9dd7a09491e8bfb34d45c1f37b1374a`；业务 smoke 源码与正式标签相同。A/B 索引只是结果集合，不将 A 组失败隐藏为整体通过。

## 本机准备异常与磁盘影响

- WSL 空闲退出使临时 loop 挂载消失；启动前检查拒绝继续。未格式化旧盘，确认原自有 ext4 后重新挂载，并使用 30 分钟有界前台进程保活，没有修改 WSL 全局配置。
- 首次 skopeo 转换没有遵守 `TMPDIR`，在 WSL 默认 `/var/tmp` 生成了临时大 tar。发现后停止自有准备 unit；精确临时文件确认已删除，D 盘已完成的 OCI 布局保留。续阶段复核全部输入/层，并将该进程的 `/tmp`、`/var/tmp` 和容器缓存目录绑定到 D 盘临时 ext4，实际挂载已确认。
- 该错误使 C 盘可用空间从约 24.5 GiB 降到约 20.9 GiB。Ubuntu 虚拟磁盘位于 C 盘且不是稀疏文件；删除内部临时文件不等于 Windows 物理占用已回收。没有擅自终止 WSL、修改稀疏设置或压缩用户现有虚拟磁盘。
- 首次运行器在委派子 cgroup 控制器时失败，未启动 Reader；补齐 Delegate / AllowedCPUs 后继续。随后 umoci 继承控制器的 umask 077，使合成根目录为 0700，UID 10001 无法执行入口。旧解包保留，重新以子进程 umask 022 解包，根目录为 0755，写时复制上层继承该准确权限；没有递归放宽用户目录权限或改软件层。
- 重解包完成后，同名准备回执的独占创建失败；没有覆盖旧文件。独立只读验收重新核实归档、原 config、全部 14 层、JAR、关键权限及包版本后生成最终 accepted 回执。这些准备失败与产品业务测试分开记录。

## 已清理的临时空间

所有 Linux 测试已结束，生成测试日志与回执复制到上述 `runtime-evidence` 后，确认自有容器库存为空、服务主进程均为 0、无嵌套挂载，再卸载并分离临时盘。30 分钟有界保活进程也已退出；Windows 本机演示继续保留。

仅删除两个本轮新建文件：16 GiB 的 `D:\Reader-Pro-Local-Image-610-20261015-a\scratch.ext4`，以及 2,174,950,743 B 的重复外层 ZIP。释放 19,354,819,927 B（约 18.03 GiB）；D 盘可用空间约 39.38 GiB。保留已核验的 `reader-image.tar.gz`，可重建测试镜像；生成样本可重新生成。清理回执为 `D:\Reader-Pro-Local-Image-610-20261015-a\cleanup-receipt.json`。

C 盘收尾可用空间约 20.71 GiB，早前 WSL 非稀疏虚拟磁盘扩张所占的 Windows 物理空间尚未回收。没有将内部临时文件已删除说成 C 盘已经完全恢复，也没有擅自压缩或关闭用户 WSL。

后续 A/B 使用新的 D 盘隔离目录，从第一步就把转换临时目录全部绑定到 D 盘。完成后保留小体积日志与回执，卸载并删除该轮独占的 16 GiB `scratch.ext4`；本轮收尾 D 盘可用约 39.37 GiB，C 盘约 20.67 GiB。正式软件归档、Windows 演示与原始输入继续保留；没有遗留该轮 Linux 测试进程。

## 保全与交付边界

本机验收文档提交 `db6cca500fbe0de8116f591bc98ed1c5ed8ba004` 触发的 [GitHub 托管 CI 38689262880](https://github.com/warpdotsys/reader-dev/actions/runs/38689262880) 已成功。实际 runner 为 GitHub 托管 Ubuntu 24.04；首次 JAR 日志摘要为上述 `b495…ee0`，第二次无构建缓存的 clean 构建与字节比较步骤均通过。本轮只核 API / 日志，没有下载解析其中的小型 XML / JSON 归档，因此不从绿灯推算精确测试数或独立报告第二份摘要。该 run 不发版、不部署；其后本机标题补查文字属于本地追加记录。

58 份用户保护报告仅核哈希；原始 JAR、只读备份及唯一授权 EPUB 的摘要不变。Git index SHA-256 仍为 `a14737af619e9c74029336262bf58bc11e10796d41f9272bedd2b03eb5ebbfb0`。没有读取私有正文、导入个人浏览器凭据、修改 Go/Rust 工程或生产数据。没有为本机测试重发标签、重建正式版本或重新部署。演示和压力测试均在隔离配置中关闭在线许可证检查，不能据此判定许可证中心可用。

生产 SSH 阻断及实际生产登录 / 回滚未验收状态见 [正式交付记录](DELIVERY-V6.1.0-2026-10-15.md)。本机通过不替代生产验收，不代表完整 Goal 完成。
