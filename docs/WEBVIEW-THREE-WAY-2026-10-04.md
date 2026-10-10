# 无外网 WebView 三方差分

## 范围与结果

用户明确扩展了 `/var/tmp/reader-generated-fragments-20261003.RMZay2` 内两个 JAR 副本的用途，允许无外网的生成 WebView 三方测试。本轮没有上传起点凭据、真实正文或生产数据；没有改动生产服务。

**初轮业务差分通过但资源门禁失败；同日两轮分阶段复跑通过有限样本门禁，最高峰值的余量仍很小。** 下述初轮三条链路各执行 5 次搜索，共 15 次；业务探针退出码为 0。完整 HTTP 状态、原始 `ReturnData` 对象、14 个书籍字段及默认值逐项相同，包含第 4、5 次脚本改写，以及目标端实际观测到的 GET/POST。不是将不同时期、不同夹具的结果拼成三方。分阶段结果与限制见本文追加节，初轮失败证据不覆盖。

| 链路 | 实际输入 | 目标 Cookie 序列 |
| --- | --- | --- |
| 原始 JAR → 固定历史 WebKit | 原件副本 SHA-256 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c` | 五次均为空 |
| 恢复 JAR → 同一历史 WebKit | 已授权副本 SHA-256 `74fbd5a5a4b77a78352b54e82dc446590197eb99ffe2a96495acff7834084306` | 五次均为空 |
| 同一恢复 JAR → 内置 Camoufox | 同一 JAR、同一目标夹具、独立生成账号和存储 | 空 → `session=alpha==` → 空 → 空 → 空 |

上述 Cookie 是夹具生成值，不是起点凭据。内置引擎回放和删除 Cookie 是有意保留的行为改进，**三方 Cookie 行为并不等价**；不能仅以其余 JSON 一致声称所有行为兼容。

每侧前四次目标请求为 GET；最后一次为 POST，正文 `q=post`、测试头 `X-Fixture=synthetic`。第 4、5 次目标 HTML 中的书名必须经 `webJs` 改写才能通过 Reader 解析断言。记录原始 `ReturnData`，不将缺失字段补成空字符串；严格 JSON 比较不将布尔值与整数视为相同。

原始小型证据：

- [实际完整三方返回](evidence/webview-three-way-generated-2026-10-04.json)
- [实际共享资源和清理记录](evidence/webview-three-way-resources-2026-10-04.json)
- [镜像配置与清单标识链](evidence/webview-three-way-image-identity-2026-10-04.json)

下载至本机的服务器原报告 SHA-256 为 `20f7014f12b09ba0bfe6b6ea8ca1fc4ce2020018cf4e2c22ec6ff7799f628d44`；原资源记录为 `63e18e43faf2061a75e3fd649d0b1e9cb49e77f9574e536601b3fd4392355ff6`。入库 JSON 保留相同语义，统一格式，不冒充服务器原文件的逐字节副本。

## 隔离与资源事实

历史服务固定为 `hectorqin/remote-webview@sha256:b61d8e86f69a743baa06aadb58cdba6d1e11c5baa45cec05a908d541ce9a684a`。其容器 `--network none`，另一个非 root 测试容器共享该私有网络；只存在 `lo`，不发布任何端口、不挂生产目录、不使用 host PID 或特权容器。

宿主 root 仅改变自己的网络命名空间后执行既有守卫，此时 `/proc/1` 仍是宿主 PID 1，确认网络不同且只有回环接口；将实际 inode 传入降权探针，由探针再次检查。没有为绕过容器内 PID 1 的差异而放松既有守卫。

两个容器与所有 Java、浏览器、夹具进程位于同一个新建 systemd slice。实际共同上限为 **2 CPU / 2 GiB / 禁止 swap / 512 任务**；每个子容器另外限制为 256 任务。不是给两个容器各 2 GiB 就宣称总计 2 GiB。

本次实际 `memory.peak=2147483648`、`memory.events.max=4007`，`oom=0`、`oom_kill=0`，进程限额事件为 0。硬上限如约生效，但出现内存回收压力，因此现行严格资源门禁拒绝把本轮记为无压力通过。业务探针的 `probeCompleted=true` 仅表示 15 次业务断言通过，**不是整个测试被接受**。后续工具明确分别记录 `budgetAccepted` 和 `overallAccepted`，不能修改旧证据或忽略 `max` 计数以制造绿灯。

两个专用测试容器均已按本轮随机标签及精确 ID 清理，临时 slice 已停止；两个 JAR 哈希未变。诊断和报告保留在指定临时目录。三项生产容器仍运行，没有停服或读取生产正文。

## 运行环境标识差异

Camoufox 运行环境来自 [5b56bacb 的 GitHub 原生演练](https://github.com/warpdotsys/reader-dev/actions/runs/37187152865)，amd64 制品 ID `11297951137`。归档直接下载到服务器，不经本机下载或压缩；SHA-256 实际匹配 `ad30339a93f1e69db243533ea924189c499fee150cc9c8d4356d94801499bce3`。

首次传入 CI image ID 时，服务器检查失败，**未启动原 JAR**。进一步检查发现 CI 记录的是镜像 config digest `ea2814b82bba7b3318347797ed93a7f5937437c5572276a6f3a9a18bb3ae56b2`；本机服务器 Docker 29.7.2/containerd store 的实际 image ID 是 OCI manifest digest `190e971278aa1530536b3c635d8bdeb344f0101e445bd2c61750f4417a62ffaf`。归档中实际存在后者，实测该 blob 的 SHA-256 正确，且其 `config.digest` 正是 CI 的前者。确认这条内容寻址链后才用服务器的实际不可变 ID 启动；没有将两个不同摘要当作相等或直接跳过检查。

镜像自身的 5b56bacb JAR **不是本次受测后端**：运行时只读挂载此前获准上传的恢复 JAR，三个后端输入哈希如表。不能由本次结果推断所有后来构建的 JAR 都已通过原件差分。

## 可重复命令与当前限制

在已经准备固定镜像、四份相邻 Python 脚本和只读 JAR 副本的 Linux/systemd Docker 测试主机上：

```bash
python3 scripts/run-three-way-webview-in-docker.py \
  --original /absolute/read-only-original.jar \
  --restored /absolute/authorized-restored.jar \
  --runtime-image sha256:190e971278aa1530536b3c635d8bdeb344f0101e445bd2c61750f4417a62ffaf \
  --output /var/tmp/reader-generated-webview-NEW
```

输出目录必须是新目录；资源或业务门禁失败时返回失败，保留生成诊断并清理自己的测试容器。当前脚本采用下文的分阶段交接；原来并存实现的失败仍保留，不据此预测所有后续运行都会成功。Python 单元测试只验证拒绝条件、完整报告比较和实际资源字段读取，不能冒充 Java/浏览器实跑。

尚待完成：资源余量与长时负载、真实需登录书源的同条件三方测试；原生产远程引擎版本溯源；完整脚本、代理、编码、超时和长期并发边界。起点网页 Cookie 验证另见[登录记录](QIDIAN-SOURCE-LOGIN-2026-10-04.md)，不是服务器测试的第三方凭据。

## 分阶段资源复跑（同日追加）

新工具先完成原件和恢复版的历史引擎各 5 次实际请求，严格验证完整 JSON、目标 GET/POST 和空 Cookie 序列，然后发出交接标记。宿主只停止有本轮随机标签的历史测试容器；另一个容器继续持有同一网络命名空间和同一个目标夹具。宿主重新执行只回环守卫，探针再次复核 inode/接口并确认旧端口 8050 拒绝连接，之后才允许内置引擎执行余下 5 次。不是分别运行三个不相同的样本再拼报告，也没有用预期结果代替历史侧。

所有后端依然顺序运行，JVM 堆参数仍为 `-Xms128m -Xmx768m`，两个 JAR 和运行环境的字节标识与初轮相同。共享 **2 CPU / 2 GiB / 无 swap / 512 任务** 不变；没有重置累计 `memory.peak` 或事件，没有清空宿主缓存，也没有忽略触限计数。交接失败、旧端口仍存活、标记异常或资源触限都会拒绝成功。

服务器 `run-c-phased` 真实运行退出码为 0，15 次完整业务断言全部通过；`probeCompleted`、`budgetAccepted`、`overallAccepted` 均为 true。总内存峰值 **2,127,847,424 B（约 1.98 GiB）**，`memory.events.max/oom/oom_kill` 与任务触限均为 0。两个 JAR 哈希未变，两个本轮容器已经按精确 ID 清理，生产服务仍运行。

- [分阶段实际完整返回](evidence/webview-three-way-phased-generated-2026-10-04.json)
- [累计限额、86 次采样、交接及清理记录](evidence/webview-three-way-phased-resources-2026-10-04.json)

对应服务器原报告 SHA-256 分别为 `9cfafef3e35f7a7dac2f190dcb16ba1bde5549cfc4bb9b22e4de83450bf648f8`、`90668580eae3adccb2e2837be4b9e216fd312f494d6b8e84faf56d8a510269c8`；本机下载逐字节复核后再次用比较器验证三侧实际数据。入库 JSON 格式规范化，不冒充原始字节哈希。

采样实际看到历史引擎停止前/后匿名内存从 65,388,544 B 降至 14,827,520 B，但文件缓存计数仍为 765,636,608 B。内置阶段最高采样点约 2.09 GB，其中匿名内存约 0.70 GB，`file` 计数约 1.37 GB；`shmem` 属于文件统计的一部分，不能重复相加。采样峰值不是内核累计峰值，且本轮没有控制宿主缓存温度。由此只确认交接释放了不再使用的进程并让此轮门禁通过，**不能唯一归因初轮压力、宣称冷启动或长期稳定**。

本轮距离 2 GiB 上限只剩约 19 MiB，仍是发布前风险。单个完整镜像的 CI 预算报告是另一种运行拓扑，不能与同时含历史参考的共同 slice 混用。当前后端/界面没有因这次测试修改生产内存设置。Python 单元测试在 Linux 为 76 项全部通过；Windows 为 75 项通过、1 项 POSIX 符号链接用例跳过，测试本身不启动 Docker、Java 或浏览器。

### 第二轮独立复跑

在同一服务器、相同只读输入及同一脚本上，以全新的存储、账号、夹具、测试容器和 slice 执行 `run-d-phased-repeat`。该轮实际退出码为 0，三侧各 5 次完整请求再次通过；三个接受标志为 true，总峰值 **1,952,653,312 B（约 1.82 GiB）**，内存触限/OOM/任务限额事件均为 0。两个 JAR 哈希不变、精确 ID 清理完成，生产未改。

| 实际运行 | 业务请求 | 总内存峰值（B） | `memory.events.max` | 整体接受 |
| --- | ---: | ---: | ---: | --- |
| 初轮并存 `run-b` | 15 / 15 | 2,147,483,648 | 4,007 | 否 |
| 分阶段 `run-c-phased` | 15 / 15 | 2,127,847,424 | 0 | 是，仅有限样本 |
| 分阶段复验 `run-d-phased-repeat` | 15 / 15 | 1,952,653,312 | 0 | 是，仅有限样本 |

新增[完整业务结果](evidence/webview-three-way-phased-repeat-generated-2026-10-04.json)和[81 次实际资源采样及清理记录](evidence/webview-three-way-phased-repeat-resources-2026-10-04.json)。服务器原文件 SHA-256 为 `989d77c114f8a4d95cac79a9b9193e5d6bd456e8955ae49a7da32800e67f836b`、`eb7a53b4d3c40f0b8a9c1d85661ad109e71c721f8efb57e66d354e8dd855e11d`；本机下载哈希匹配，并由严格三方比较器再次检查，不根据上一轮补值。

两轮通过不抹去初轮失败，也不抹去最高峰值余量风险；未清空宿主缓存，不把资源变化全部归因于交接。没有真实登录凭据、站点正文或生产目录加入任何一轮。服务器运行的探针字节哈希为：`compare-webview-cookie.py` = `22ca3d0926692c12be69d89e9dee9d608f2b4882bdaaa7b1abf464fcba4d6fb3`，`run-three-way-webview-in-docker.py` = `f52a93ece08070709fe94f768d5bb76c6bbd9783e7ec8e69cc97cca073661664`；均在传输后与本机实际文件核对，不以以后脚本覆盖已保存结果。
