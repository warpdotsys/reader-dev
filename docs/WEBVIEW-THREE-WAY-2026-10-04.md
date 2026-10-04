# 无外网 WebView 三方差分

## 范围与结果

用户明确扩展了 `/var/tmp/reader-generated-fragments-20261003.RMZay2` 内两个 JAR 副本的用途，允许无外网的生成 WebView 三方测试。本轮没有上传起点凭据、真实正文或生产数据；没有改动生产服务。

**业务差分已实际通过，整体资源验收未通过。** 三条链路各执行 5 次搜索，共 15 次；业务探针退出码为 0。完整 HTTP 状态、原始 `ReturnData` 对象、14 个书籍字段及默认值逐项相同，包含第 4、5 次脚本改写，以及目标端实际观测到的 GET/POST。不是将不同时期、不同夹具的结果拼成三方。

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

输出目录必须是新目录；资源或业务门禁失败时返回失败，保留生成诊断并清理自己的测试容器。不能把这条命令目前的资源失败解释成预期成功。Python 单元测试只验证拒绝条件、完整报告比较和实际资源字段读取，不能冒充这次 Java/浏览器实跑。

尚待完成：调查内存触限原因、减少不必要的对照进程并存并重新验收，不能将原因尚未确定的压力写成已修复；真实需登录书源的同条件三方测试；原生产远程引擎版本溯源；完整脚本、代理、编码、超时和长期并发边界。起点网页 Cookie 验证另见[登录记录](QIDIAN-SOURCE-LOGIN-2026-10-04.md)，不是本轮服务器测试的第三方凭据。
