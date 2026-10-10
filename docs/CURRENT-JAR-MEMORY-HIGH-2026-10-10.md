# 当前 JAR 的本机资源诊断与显式提前回收验收

## 已验证身份与范围

2026-10-10 实际下载 Native [37946438729](https://github.com/warpdotsys/reader-dev/actions/runs/37946438729) 的共享 JAR 制品 `11624549044`。源码为 `c594217119aae2bc48bf8792e06306995d69a672`，受测合并快照 `0db121d5ab611cef4d32185e1333066b081ca9f5`。下载目录包含 JAR 和 `SHA256SUMS` 两个文件；首次下载后的“只有一个文件”断言失败被保留，没有重新下载。随后只读校验文件集合、校验单原字节、完整 JAR SHA：`e59a01bf190de757f73216557ff353411019507d25bad76f0c808b48366adb84`，285,665,012 B。外层 ZIP 未在本机独立复算 SHA，不把 JAR 散列冒充 ZIP 散列。

这些测试**不是最新完整 Native 镜像的本机验收**。只将上述 JAR 的独立校验副本只读挂入已经验过的 8d／3fe runtime：image ID `sha256:401d895c31a39634bf29654a7fbb29527272d287c8c1219b12794bbe1970bc44`，原镜像 JAR `591ff…dcec` 保持不变。有关浏览器、Dockerfile、入口和生成 smoke 的源码与 8d 相同，不等于两个镜像的全部字节相同。最新完整 AMD64 image ID 仍是 `sha256:b9576c5f67f062f09239460a2edc107392a0a8ddf1306547f9ec5ae497aeaf70`，未重新下载其大归档。

全部轮次只用生成书源和账号：私有网络只有 loopback；容器 UID/GID 10001，附加组为空、能力全零、禁止提权；宿主独立核对 PID、cgroup 和网络后才授权原入口启动。父级统一 2 CPU／2 GiB／256 PID／零 swap，子级限额不能替代父级判断。主进程退出码、PID1 回收、独立 stopped/pid0、容器删除后清单分别核对。无真实 Cookie、书籍正文、生产数据或生产服务变化，没有停止或重启 WSL。

## 实际结果：保留失败，业务与资源分开

每轮 Vue3 首页／登录／搜索／生成阅读入口的静态字节与 JAR 对照通过；14 次生成渲染覆盖搜索、旧式脚本、POST/头、异步、Cookie 删除隔离、延迟详情及四账号并发。主测试退出 0，浏览器/worker/driver 均为 0，清理后自有容器剩余 0。**这不是页面实际渲染、真实登录、原始 JAR 三方或真实站点通过。**

| 轮次 | 唯一诊断变体 | 父级内存峰值 B | high / max | PID 峰值 | 总时长秒 | 整体 |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| 10-c | 原 Xms256m/Xmx768m | 2,147,483,648 | 0 / 1816 | 219 | 51.030 | 失败 |
| 10-d | 仅 Xms128m，Xmx 不变 | 2,147,483,648 | 0 / 881 | 225 | 42.067 | 失败 |
| 10-e | 仅 worker 退出后建议回收镜像字体的干净页 | 2,147,483,648 | 0 / 788 | 222 | 50.010 | 失败 |
| 10-f | 原堆/原 Python/原字体，父级 memory.high=1.5GiB | 1,611,399,168 | 2893 / 0 | 218 | 62.450 | 通过 |
| 10-g | 相同提前回收配置，独立新目录重复 | 1,611,395,072 | 3768 / 0 | 220 | 44.503 | 通过 |

五轮 OOM、OOM-kill、PID 触限及 swap 均为 0；前三轮即使业务成功也严格失败。10-e 实际对 14 次退出后的 641 个只读字体文件给出缓存建议，没有删除/改写字体，但首次加载仍触限，因此不作为正式修复。两次提前回收保留所有浏览器功能、请求数量、字体、Java 堆和原硬预算；没有外部预热、全局清缓存、重置峰值/计数或提高上限。每次当前 JAR 拷贝/散列与局部干净页建议都在同一父级预算内。

原始各轮报告的完整 SHA、失败与身份见[小型机器记录](evidence/current-jar-memory-high-local-2026-10-10.json)。原 [8d 镜像失败链](LOCAL-LOCKED-IMAGE-2026-10-08.md)不改记为成功。

0.1 秒采样只读自身 cgroup 的数字；10-c 的一个最高实测样本中 anon 789,127,168 B、file 1,331,785,728 B，支持继续调查文件页压力，但不是精确峰值 RSS 分解或唯一根因。shmem 包含在 file 中，类别不可相加。两次成功只证明有限生成样本；时长差异不支持吞吐、无泄漏或长期稳定承诺。

## 可维护的 GitHub 入口与独立托管结果

现有 `browser-image.yml` 的离线 soak 新增 `soak_memory_policy`：默认 `unchanged`，显式 `high-1536m` 才创建本次独有随机 systemd slice，在 Reader 启动前配置 `MemoryHigh=1610612736`，仍硬限 2 CPU／2 GiB／256 PID／零 swap。必须是 GitHub 托管 runner、cgroup v2/systemd Docker 驱动，容器真实宿主归属核对后才进行测试。不安装宿主浏览器或改变镜像。

`scripts/reader-browser-budget-slice.sh` 单独保存启动前、运行后、退出前父级资源、实际归属和删除后 slice 清理。旧的容器内逐轮、故障恢复、JSON/Cookie 和资源门禁不放宽；新比较器另外核对父级身份、完整原硬预算、累积峰值/high 计数、清理结果。父级 max/OOM/PID 触限、未知限额、非零 swap 或未证实清理都会失败，不能用叶子组通过掩盖。失败报告保留；不会盲目停止仍 populated 的 slice。

`report-browser-cgroup.py` 增加只读 `memoryHighBytes` 和有界 cgroup 目录入口；历史报告缺少此字段保持未知，只有显式选用新政策才要求实际值匹配。`memory.high` 是提前回收/节流边界，可短暂超过且不直接触发 OOM；`memory.max` 才是硬限制。高压力事件如实保留，不能声称它们为 0。应持续监测延迟，过紧会影响性能。[Linux 内核官方说明](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory)。

本地新 13 项生成回归通过；Windows 全套 413 项通过、1 项原平台跳过，发布身份 Node 19 项通过。WSL 首次全套因缺少 Node 失败两项，记录保留；随后复用已下载镜像的原 Node 作为测试环境，实际 413 项全执行通过／16.561 秒，不安装软件、不跳过 JS 语义检查。shell 语法和发布流水线静态检查通过；自己的托管长期结果另行记录，不能以合成回执单测冒充真实 systemd/Docker 执行。

可用现有托管入口复用本次原完整镜像，不重新构建它：

```powershell
gh workflow run browser-image.yml --ref ci/full-reader-20260926 -f native_arch=amd64 -f soak_native_run=37946438729 -f soak_revision=0db121d5ab611cef4d32185e1333066b081ca9f5 -f soak_seconds=600 -f soak_memory_policy=high-1536m
```

该命令会在托管 runner 下载约 2 GiB 原镜像，不在本机重复下载，不传私有 JAR/会话/正文，不发布或部署。执行时间、来源和结果必须按实际运行回填；新工具源码身份与被消费的 c594 镜像身份分开。

### 第一轮托管发布门禁失败及精确修正

工具源码 `f2dff030640af21daddcd14fb93101c309408c27` 的 [Java38005979632](https://github.com/warpdotsys/reader-dev/actions/runs/38005979632) 与 [Native38005979602](https://github.com/warpdotsys/reader-dev/actions/runs/38005979602) 实际在发布门禁负面测试中失败，未进入 Gradle/JAR/镜像构建。新增 `report-browser-cgroup.py` 文件触发规则后，原比较器在 YAML＋脚本混合文字里匹配到文件名，导致删除真实资源命令的用例被误分到后面的 async 门禁错误。不能借父版 c594 构建或本地单次静态检查宣布这版 CI 成功。

本地完整复现原 49 项中的 1 条失败；再加三条负面回归，实际 52 项／4 失败：原错误原因，以及被注释、忽略失败、重复覆盖回执的资源命令都没有被准确拒绝。修正只要求 smoke 脚本中恰好一组真实、失败不可忽略的 collector／receipt 命令；YAML 路径或注释不能充当执行证据，不删用例或放宽原错误断言。修正后发布门禁 52、原生身份 19、manifest 14，共 85 项全执行通过。自己的新托管结果仍须另验。

[独立离线专项38005988689](https://github.com/warpdotsys/reader-dev/actions/runs/38005988689) 已实际失败，作业 `114074785422` 属于 GitHub 托管 `ubuntu-24.04`，其余七项互斥任务跳过。前述 c594 完整镜像实际下载、加载且两阶段身份匹配；停止在资源配置前置判断，没有启动 Reader、进行 600 秒业务长测或取得内存/清理回执。只下载 1,009 B 小制品并逐字段/SHA 核对三个 JSON，不重新下载大镜像。[失败的有限记录](evidence/memory-high-hosted-f2dff030-2026-10-10.json)。不能把成功加载写成运行通过。

原判断要求不存在的 slice 显示 `LoadState=not-found`。本机对独有未运行名称只读核对，实际 `loaded/inactive`，FragmentPath/DropInPaths/ControlGroup 为空、Transient=no、cgroup 不存在；同名 service 才是 not-found。由此确认该 slice 前置条件有误，但首次托管没有采到这些值，不能事后补造或唯一归因。改为核对无配置、无活动状态、无 cgroup 的隐式 slice，仍拒绝任何活动/配置/drop-in/transient/归属记录；现有工作负载不能借此被覆盖。三个新增生成回归包含实际 Bash 状态谓词和无副作用 CLI 拒绝，全部通过。

新增有限只读 host preflight 位于两个制品下载之前；不支持 cgroup v2/systemd 时先保存失败状态再拒绝，不再为这种前置失败下载大镜像。正式门禁必须绑定该实际观察、父级资源及删除后的清理。宿主回执目录由 runner 持有并加 sticky 位；UID10001仍可写自己的生成业务回执，但不能移除/替换宿主独立观察文件。容器归属阶段另核对这些权限；原无外网、能力、限额和业务门禁不改。专项脚本源码与后续发布门禁修正分别记录，修正工具自己的托管结果仍待验。

最终修正后本机再次执行：Windows全套416项／23.549秒／1项原平台跳过，WSL全套416项全执行／17.481秒；发布相关85项在两系统全执行通过。新Bash谓词/CLI检查确实执行，但没有在这些生成单测里启动systemd或Docker，真实runner的前置/启动/清理仍必须单独验证。初版413结果和全部失败保留，不用测试环境修正覆盖历史。

### 修正工具自己的托管结果已取得

工具源码 `32bb66e587d7e06e43f157a5f7963a49e23049f3` 的 [专项38007262699](https://github.com/warpdotsys/reader-dev/actions/runs/38007262699) 已实际成功，唯一执行作业 `114078808182` 属于 GitHub 托管 `ubuntu-24.04`，其余七项互斥任务跳过。只下载 10,246 B 小报告制品 `11651844540`，逐字段、逐轮和原始文件完整 SHA 独立验收，不在本机重新下载大镜像。工具源码与消费的 c594／0db 完整 AMD64 镜像分开记录；此轮不是新 32／541 Native 镜像的长测。

真实下载前 preflight、启动前父级限额、容器归属、运行后与退出前资源、容器移除及 slice 清理均取得。连续 612.408 秒／51 轮／212 次成功搜索，三类故障后恢复、Cookie／请求形状错误 0；父级峰值 950,427,648 B、叶子峰值 950,411,264 B、PID 峰值 212，保持 2 CPU／2 GiB／256 PID／零 swap，max／OOM／PID 触限为 0。实际 high 计数 **0**：证明配置和门禁确实执行，不证明这轮触发了提前回收，更不能把与本机不同的峰值归因于该政策。宿主独立观察容器移除后剩余 0，slice populated=0、两个自有单元 inactive；不以测试脚本自己退出替代清理。

静止阶段内存仍增加 61,468,672 B，最慢观测请求 16.005 秒；有限生成压力通过不证明没有泄漏、容量或生产长期稳定。上述指标继续作为已知风险保留。[此次托管的有限机器记录](evidence/memory-high-hosted-32bb66e5-2026-10-10.json)。初版 f2 的实际失败和本机前三轮父级触限失败保持原结论。

同一工具源码自己的普通四流水线亦分别核对实际小报告：

- [Java38007232983](https://github.com/warpdotsys/reader-dev/actions/runs/38007232983)：43 个 JVM suite／181 项／31 项原环境跳过，编辑 11 项全执行；Python416项全执行。两次干净构建同 e59 JAR／285,665,012 B／1,569 条目，内容与 ZIP 元数据差异 0。工具变化没有改产品 JAR；不是原始 b26 兼容性证明。
- [Vue38007232992](https://github.com/warpdotsys/reader-dev/actions/runs/38007232992)：25 项实际 Chromium 页面流程、零跳过，296 前端＋20 截图守卫、类型构建通过；11 图尺寸／散列核对，四张编辑成功／重名／过期／离线图实际目视中文可读，仅此范围，不宣称整个界面无编码缺陷。
- [Full38007233107](https://github.com/warpdotsys/reader-dev/actions/runs/38007233107)：Camoufox20／Chromium23／helper3及异步、详情、默认UI门禁分别通过，同 e59 JAR，峰值 829,030,400 B／PID195。普通短测实际 swap0，但配置允许1GiB，不能当作零swap长测。
- [Native38007232999](https://github.com/warpdotsys/reader-dev/actions/runs/38007232999)：共享JAR、原生双架构、重新导入后运行、正式发布导入器六个实际作业全成功；24份publisher JSON、身份及原守卫独立核对，同 e59 JAR，最大峰值875,286,528 B／PID205。受测快照 `541892bde39c2473ac121e743b294e93f3e6825d` 与源码32只多合并提交、文件差异0；AMD64 image `sha256:22063372687203d08cc278f40245da5b71967f04b266991a2f7c19bd222d3430`，ARM64 image `sha256:558f34dc63a838fd242a4464c0595d970e3adbf9de96632d6172fce7cbc41109`。本机未下载两份新大归档，不冒称归档SHA已在本机复算或整镜像本机通过。

三个独立普通验收摘要 SHA：UI/Java `e1a4c948e9b21ac4ee23a3594993034118eb1151470d4f8f0ad92a19a6030f9f`，Full `ba81d057cacc1e5c1a9a6ded1deafd21700cce542acd30292cab40a810f45d5c`，Native `74d5d9cede3d9e834a59d4e246b2ae56b1490e0e0f3ae2c98b20138d6c3367e4`。专项独立摘要 SHA `8976dadb16320cbb1fa46aae4adefdb9f4f5a220f390747e864a436c82f808bb`。摘要不是原始回执或大归档的 SHA。

`uidmap` 只读再验为 `1:4.17.4-2ubuntu3`，包文件校验无差异；subuid/subgid 两份完整散列仍 `d796e52bc335df4e55114fad949f19850e6b4008cf07bf8d53a4e88936be9cbd`。只读查询前 Ubuntu 显示 Stopped，正常启动查询后自有10-g单元 inactive/MainPID0；本轮没有重复安装、停止或重启WSL、修改映射、访问生产或读取真实凭据/正文。58份用户报告及原始b26 JAR再次完整散列核对未变。

## 尚未完成与回退

- 最新完整 Native 镜像的本机验收、准确 b26 原件的当前三方、真实认证书源，以及新32／541镜像或更长时段的持续验收仍未完成。本次只关闭修正工具消费c594整镜像的有限612秒生成压力门禁。
- 起点匿名详情、历史 UTF-8 POST 差异及旧编码问题不被本次生成成功消除。Vue3 真实登录/阅读与全界面边界仍需逐项验证。
- 没有改生产 Compose、Java 堆、字体、指纹平台或浏览器资源；该政策只作显式验收候选，不强制到生产，更不表示 PR 已合并、发版或上线。
- 不选 `high-1536m` 即保持原 soak 路径。只清理本轮独有随机 slice/容器，保留失败与原字节产物，不回退用户存储。原始 `D:\Download\reader-pro-3.2.14.jar` 本轮只读再验仍为 b26 完整 SHA；58 份用户报告完整散列全匹配，未提交/覆盖它们。
