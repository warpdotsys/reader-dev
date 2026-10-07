# 高核数本机的数值线程池：已修复因素与未解决的资源风险

## 实际发现，不等于全部根因

按授权安装的 Ubuntu 官方 `uidmap`／必需 `libsubid5` 均为 `1:4.17.4-2ubuntu3`，`newuidmap`／`newgidmap` 可用。未修改代理、原 subuid／subgid 范围或生产。后续探针继续使用 UID/GID 10001、无有效 capabilities、no-new-privileges、独立自有 cgroup 的 2 CPU／2 GiB／256 PID／零 swap。

WSL 向进程暴露 32 个 CPU。锁定的 NumPy 2.2.6 使用 OpenBLAS；只导入 NumPy／Camoufox 0.5.6 就使同一个新 Python 进程的线程数从 1 变为 32。另一个新进程在导入前将数值库线程选项设为 1，导入后仍为 1。该离线测量实际 2.922 秒、峰值 198,729,728 B／PID 46，原零 swap 守卫重新执行通过；没有页面、Reader、账号或正文。

两个导入进程的最大 RSS 为 100,792／100,720 KiB，几乎相同。因此只确认减少 31 个导入线程，不能写成修好了整页内存、Firefox 全部线程池或起点解析。CPU quota 也不能假定等于自动线程池所见核数；[Mozilla 的 Linux CPU 枚举实现](https://raw.githubusercontent.com/mozilla-firefox/firefox/main/nsprpub/pr/src/misc/prsystem.c)另读系统 CPU 信息，但这不是本次请求唯一根因的证据。没有设置未经证实的 Firefox 环境变量、关闭 site isolation／sandbox／JavaScript 或增加资源限额。

## 可读源码修改及真实门禁

`src/main/resources/camoufox/worker.py` 在第一次 Camoufox／NumPy 导入之前，显式设置 `OPENBLAS_NUM_THREADS`、`OMP_NUM_THREADS`、`MKL_NUM_THREADS`、`NUMEXPR_NUM_THREADS` 为 `1`，覆盖继承的高核数或非法值。只作用于 Reader 的一次请求 worker；不改系统环境、协议、依赖版本、ReturnData、Cookie、上下文隔离、网络防护或超时。

四个纯测试覆盖缺失选项、32／64／非法继承值、无关选项不变及真实 Python 模块导入顺序。新增 JVM 原生契约 `numericalLibraryImportsDoNotAllocateTheHostCpuThreadPool` 从打包资源加载 worker，强制继承四个 `32`，要求锁定版本导入后的真实 `/proc/self/status` 线程数为 1。它有自己的 20 秒进程截止／30 秒测试截止、4 KiB 有限报告和自有进程回收，不输出原始诊断。

正式契约集合从 18 增为 **19**，原 18 个语义不变。守卫明确拒绝旧 18 项、缺项、跳过项及伪造零计数；生成 XML 的纯测试只是守卫检查，不是原生运行证明。新增 JVM 契约本机已编译，实际执行须由本次提交自己的 GitHub 托管原生／完整镜像工作流验收，不能借用 `c3307ded` 的旧报告。

## 已成功重建与有限本机运行

- 全量 Python 实际 231 项／13.579 秒：230 通过、1 个 Windows POSIX 夹具跳过，失败／错误 0。
- 发布结构、原生制品身份及双架构 manifest 的 69 项 Node 检查实际通过，无跳过；这是守卫及生成制品测试，不是发布或新镜像运行。
- 离线 `bootJar` 成功，随后 Java/Kotlin 主编译与 `compileTestKotlin` 成功；没有将编译说成新增 JVM 契约已运行。
- 新 JAR SHA-256 `f9c9082a352d0d609f3a82497f4d4b0ed2907da72bec833a962ce5948a184f2b`，包内 worker 与源码 SHA-256 `01863dcd45eab836bc0e76e2f0512d97656112a43c8f7dd1ffa3e3e8845087be` 一致，原 Vue 2 资源保留，Vue 3 未修改。
- 旧只读锁定运行时＋上述新 worker 在无外网 namespace 中实际跑完 3 个生成文档 helper，15.177 秒，原文档守卫及三次资源守卫独立重新执行通过，自有运行状态清除。它不启动 JAR，不是当前完整镜像／19 项 JVM／书源或三方验收。

该 helper 的峰值仍为 **2,123,980,800 B（约 1.978 GiB）／PID 200**，距 2 GiB 上限仅约 22.4 MiB。虽然触限／OOM／swap 为 0，不能因此声称完整 Reader 在本机预算内稳定；停止浏览器后仍有约 1.59 GB cgroup 记账，但未采集 `memory.stat`，不能唯一判定缓存或泄漏。它与旧 helper 的峰值不是严格控制的整页 A/B，不宣称总体内存下降。[具体导入对照、原守卫、散列和范围](evidence/camoufox-numerical-threads-local-2026-10-07.json)。

可重复构建（准备 JDK 11 和锁定前端产物后）：

```powershell
.\gradlew.bat -PreaderWebUi=vue3 bootJar --offline --max-workers=2 --no-daemon "-Dorg.gradle.jvmargs=-Xmx512m -XX:ActiveProcessorCount=2"
```

干净 GitHub runner 会先安装锁定 Node／Python／浏览器依赖并构建前端，不依赖本机旧运行时、宿主 Chrome 或独立 WebView 服务。

## 新增已知问题：本机匿名请求同时触及内存和 PID 上限

在数值线程修复之前，独立目录中的旧运行时＋恢复 JAR `eaa34cc9...`／旧 worker `2bc9fe2f...` 做过一次匿名起点详情。测试格式及六项 HTTP 保存读回均完整；唯一 Reader 监听实际确认为自有 IPv4-mapped loopback，而非外网监听。请求 6.747 秒、HTTP 200／`isSuccess=false`／data null，错误长度 66，固定类别及页面诊断均 null；不保存原错误，不猜唯一原因。

这一轮资源也明确失败：内存峰值 2,147,483,648 B，`memory.events.max=2549`；PID 峰值 256，`pids.events.max=2`。无 OOM／swap 不能抵消触限，原守卫重新执行仍拒绝。三轮此前准备失败分别没有进入实际详情，不能多计为产品请求；A 的挂载原因只作推断，B 的权限和 C 的监听观测问题是探针基础设施问题。四轮原报告均保留。

测试仅用随机 Reader 账号、新存储、匿名公开元数据；没有网站会话／章节正文，也未执行原 JAR。WSL 网络共享、显式 JVM 公网 hosts 快照和旧运行时是已记录限制，不冒充当前完整 Docker 镜像或原件／历史 WebView 三方。生成 Cookie 前后为 0，退出及保护接口符合 legacy 契约，自有 Java 和运行状态已清除。[红灯、原始计数及明确测试字节](evidence/public-metadata-local-resource-failure-2026-10-07.json)。**没有再次用新 JAR 撞实站，没有升预算取绿灯；新 worker 的实站效果尚未验证。**

## 尚未验证和回滚

本修改自己的托管运行现已完成：`09163efe`／受测 `3aec4f9e...` 的 [Java](https://github.com/warpdotsys/reader-dev/actions/runs/37580306827)、[Vue 3](https://github.com/warpdotsys/reader-dev/actions/runs/37580306818)、[完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/37580306816)、[双架构原生六作业](https://github.com/warpdotsys/reader-dev/actions/runs/37580306817)均终态成功。两份真实 Camoufox XML 各 19 项无跳过／失败／错误，新导入契约 0.616／0.615 秒，文档 helper 各 3 项的原守卫通过。24 份 publisher JSON 的身份／共享 JAR／重载 image ID 及 5 组 UI／异步／资源报告独立核对，短时最高 842,723,328 B／PID 204、触限／OOM／实际 swap 0，配置仍允许 1 GiB swap。Java 162 中 132 通过／30 门控跳过，Vue 18＋1、页首 18 组及长分组 3 组通过；不借此前 18 项绿灯。[自己的验收范围和散列](evidence/camoufox-numerical-threads-hosted-09163efe-2026-10-07.json)。

之后单独补了有限 `memory.stat` 观测并跑新生成 helper，停止后约 13.6 MiB 匿名／1496.4 MiB 文件页；不是上节旧报告的事后分类、完整 Reader 峰值分解或无泄漏证明。新 collector 自己的 hosted 尚待验收，详见[观测范围及原预算不变](BROWSER-MEMORY-CATEGORIES-2026-10-07.md)。完整新镜像本机高核数资源、真实起点认证及正文、同条件三方、长期并发和生产仍待验收。当前 PR 保持草稿，不创建正式标签、不推 registry、不部署。58 个用户报告哈希全部不变，原 JAR 哈希不变，未覆盖其他工程。

回滚在干净维护检出撤销这次数值线程策略并重新构建；保留新测试、原失败证据和 19 项门禁，让旧行为被回归明确检出。没有数据迁移或字节码补丁，不能在用户脏工作区 reset／checkout，也不能为接受旧报告删除新增契约。
