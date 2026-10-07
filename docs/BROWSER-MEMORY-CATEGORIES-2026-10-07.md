# 本机浏览器资源记账：当前分类不等于峰值或 RSS

## 修改内容和安全边界

`scripts/report-browser-cgroup.py` 新增有限 `memoryStatBytes`，只按名称读取 cgroup v2 `memory.stat` 的 14 个内存分类。读入硬上限 64 KiB；未知字段不输出，缺失文件／内核版本未提供的分类为 null，不伪造 0；已知字段重复、负值、非整数或超长值拒绝。消费守卫拒绝错误类型和字段集合，旧报告没有此可选分类仍按原预算判定。

**全部总量／触限／OOM／PID／swap 守卫保持原条件**，不减去文件页，不写 `memory.reclaim`、不清宿主缓存、不增加额度、不关闭浏览器隔离；不修改 JAR、worker、依赖或业务返回。新增 7 个纯测试，加上原 9 个，共 16 项实际通过；全量 Python 238 项／13.894 秒，237 通过、1 Windows 环境跳过、失败／错误 0。

根据 [Linux 内核的 cgroup v2 定义](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files)，这些是字节计数而非进程 RSS：`file` 包含文件缓存、tmpfs／共享内存，`kernel` 的子项及其他分类可能重叠，因此不能把全部项相加。当前分类也不能复原未采样瞬间的内核峰值。

## 已实际执行的有限观测

新独立服务 `reader-bounded-runtime-20261007-b`，UID/GID 10001、有效 capabilities 0／no-new-privileges；host PID 的实际 cgroup 成员由外侧独立确认后才允许浏览器启动。旧只读锁定运行时＋当前 worker `01863dcd...`，namespace 只有 loopback，无外网、网站会话、正文或 Reader JAR 启动。2 CPU／2 GiB／PID 256／零 swap，所有准备和旧 JAR 身份读取仍计入同一预算，不把它们搬到预算外取绿灯。

实际 14.333 秒，3 个生成文档 helper 的原守卫及 4 次原零 swap 资源守卫重新执行通过。26 次分类采样只保存最高观测 current 和最高观测 PID 两个有限样本，不保存页面、URL、错误原文或进程参数。峰值 **2,095,718,400 B／PID 202**，触限／OOM／swap 0；自有运行状态清除，服务终态 inactive/dead、MainPID 0。

| 实际阶段 | current（MiB） | anon（MiB） | file（MiB） |
| --- | ---: | ---: | ---: |
| 读取但不执行旧 JAR 的身份后 | 297.1 | 11.3 | 285.0 |
| 最高 current 采样（12.056 秒） | 1977.0 | 431.3 | 1521.7 |
| helper 与浏览器停止后 | 1518.8 | 13.6 | 1496.4 |

这是本次生成 helper 的实际分类：停止后总记账主要是文件页，而不是把整项约 1.59 GB 当作存活匿名内存。未启动 Java，不能称为 Reader 堆测量；也不能据此宣称无慢泄漏、唯一定位文件、解释上一轮起点失败、完整新镜像容量足够或总体内存已下降。最高 current 样本低于内核 peak，表格不是精确 peak 分解。先前没有分类的红灯报告不回填新数据、不改判定。[实际分类、采样、身份和重执行守卫](evidence/browser-memory-categories-local-2026-10-07.json)。

## 与已完成的线程修复托管验收分开

`09163efe`／PR 受测快照 `3aec4f9e...` 的四条正常工作流及原生六作业现已终态成功。两份真实引擎 XML 各 19 项／0 跳过／失败／错误，新增原生导入契约实际 0.616／0.615 秒；24 份 publisher JSON、两架构共享 JAR、5 组 UI／异步 API／总资源报告的原守卫已独立核对。普通 Java 162 中 132 实际通过／30 门控跳过，不算全引擎实际执行；Vue 核心 18＋子路径 1、18 组书架几何及 3 组长分组通过，320px 生成图已目视。[该提交自己的具体证据](evidence/camoufox-numerical-threads-hosted-09163efe-2026-10-07.json)。

这些报告由旧 collector 生成，没有 `memory.stat`，不能倒填分类。正常镜像短测最高 842,723,328 B／PID 204，触限／OOM／实际 swap 0，但允许 1 GiB swap；不是上述本机零 swap 预算，也不是起点、认证三方、长期或生产证明。完整镜像和原生共享 JAR 独立构建，哈希分别为 `c7f9fc5b...`／`221d41ef...`，不声称不同 workflow 字节相同。

新 collector 自己的托管集成仍待本次提交验收；它只是补齐可维护的观测，不是根因修复或新的业务兼容证明。实站解析、真实认证和原件／历史 WebView 同条件三方仍未完成。不发布／部署，不修改用户数据或映射范围。58 个用户报告哈希重新核对无变化。

回滚在干净维护检出撤销可选分类采集和其测试即可；保留原总资源守卫、原红灯及观测证据。不得通过删触限计数、扣除缓存、加额度或 reset 用户工作区回退。
