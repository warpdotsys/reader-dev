# 本机最终浏览器镜像验收准备

## 当前边界

目标是在本机隔离环境运行 GitHub 已验收的同一完整 Reader 镜像，后续真实会话仅进入回环 Reader 并在测试后清除。不能再用宿主 Chrome 的解析基线替代最终 Camoufox，也不把凭据传服务器、CI 或 GitHub。现在没有启动这个 Reader 容器、没有再次导入会话，也没有修改生产服务。

### 2026-10-08 前置条件状态更正

下文“uidmap 尚待授权／缺失”是 10-04 历史观察，不是当前阻塞。用户已明确授权，10-06 仅安装 Ubuntu 官方 `uidmap`／必要 `libsubid5`，同版 `1:4.17.4-2ubuntu3`，无升级／移除包；[原安装和生成非 root 探针证据](evidence/local-uidmap-runtime-2026-10-06.json)保留。10-08 再次只读确认两个映射工具存在、包版正确，`/etc/subuid` 和 `/etc/subgid` SHA 均仍为 `d796e52bc335df4e55114fad949f19850e6b4008cf07bf8d53a4e88936be9cbd`，没有改映射范围或重复安装。

后续生成 helper 的实际 UID 10001 和资源观察见[线程策略](CAMOUFOX-NUMERICAL-THREADS-2026-10-07.md)及[内存分类](BROWSER-MEMORY-CATEGORIES-2026-10-07.md)。这些是旧只读运行库加当时 worker，不是当前最终完整 Reader 镜像、真实认证或生产证明。该区分继续保留；原解包警告和早期失败不删除。

## 已实际检查与准备

- 新提交 `662df3d3` 的[四条托管验收及制品身份](evidence/network-diagnostic-ci-662df3d3-2026-10-04.json)已核对。待本机消费的是原生 AMD64 制品 `11305669149`，GitHub 制品约 2.17 GB；不是书库压缩或真实正文文件。
- Windows 没有 Docker，正在运行的 WSL Ubuntu 也没有 Docker、Podman 或 Java。WSL 有 user namespace 与用户级 systemd；一个无 Reader/无数据的小进程已在 2 CPU、512 MiB、零 swap 的独立 unit 中实际运行，不改变主机全局资源设置。
- Ubuntu 官方 APT 元数据中的 runc、skopeo、umoci、libgpgme45、busybox-static 只下载到本项目 `build/local-camoufox-image-20261004-a/` 并逐项核对大小/SHA-256，使用 `dpkg-deb --extract` 解包，**没有安装系统包或执行维护脚本**。工具准备 unit 实际 30.898 秒、CPU 4.605 秒、峰值 54.2 MiB、swap 0；这只是准备工具的用量，不是 Reader 资源验收。
- 包版本、SHA、准备脚本与生成的 OCI 探针保留在本机忽略目录。读取版本显示 runc 1.4.0、skopeo 1.21.0-dev、umoci 0.4.7；APT 包版与可执行程序自报字符串分开记录，不混成正式产品依赖。

## 离线消费核验的实际准备

本机忽略目录中的 `consume_ci_image.py` 只消费固定托管合并快照、先流式核对归档 SHA，再转换 OCI 并以 `umoci --rootless` 解包；不启动镜像、执行镜像中的 Python/Java 或导入会话。它分别核对 Docker config 摘要、转换后的 OCI manifest 摘要、全部 layer 的大小与 SHA、镜像内 JAR、UID 声明、入口、固定 Python 包和浏览器安装元数据。解包文件映射为本机用户不等于运行时 UID 10001，浏览器安装元数据中的资产 SHA 也不冒充本机重新拿到原 zip 并复算。

最先执行 6 项生成元数据检查和一个空镜像格式转换探针。前者验证原记录通过，以及错误制品 SHA、平台、root 用户、入口和重复 renderer 配置被拒绝；后者在仅允许 AF_UNIX 的独立 unit 内实际将生成 Docker 归档转换为 OCI，原 config 字节摘要不变，manifest 摘要单列。它没有容器、Reader、浏览器或真实数据，实际 1.625 秒、CPU 732 毫秒、峰值 32.3 MiB、swap 0。随后补充三项配置保真回归，最新共 9 项无跳过/失败/错误。[脱敏准备证据](evidence/local-image-preparation-2026-10-04.json)与完整镜像运行验收严格分开。

首轮格式探针失败于私有策略的文件名 scope。查证[上游 docker-archive 实现](https://github.com/containers/image/blob/main/docker/archive/transport.go)：此 transport 只接受空 scope。改为私有、仅允许本地 docker-archive 的策略，所有其他 transport 默认拒绝；具体源仍由固定命令和已核对 SHA 约束，不改用户全局信任策略，不读取 registry 凭据。修正后的空探针实际通过，原失败目录保留；这是本机准备脚本错误，不是产品或网站故障。

继续给同一生成样本添加 rootless 解包后，本机 umoci 0.4.7 对未压缩 OCI layer 报 `gzip: invalid header`，原记录保留。改为标准 gzip layer（级别 1）后，空归档转换及 rootless 解包实际通过：1.432 秒、CPU 534 毫秒、峰值 32.8 MiB、swap 0；解包后的 OCI 配置仍声明 UID/GID 10001，**没有执行该进程**。这使准备步骤不必等待映射工具，但不消除实际 runc 运行的权限缺口，也不把格式转换问题写成 Reader 运行故障。

完整下载已由原句柄成功结束，归档实际 2,174,869,020 字节。Windows 与 WSL 流式 SHA-256 均为托管记录的 `f54717102c900f97418d418f4c0e45028c9aaf87a6604e002d2bb49210d6667b`。完整离线转换/解包的结果见下一节；没有启动 Reader。

## 完整镜像离线核验已完成，实际运行未验收

首次完整转换实际 288.660 秒，因 config 摘要从原 Docker ID `093dd1c3…` 变成 `dbe46a6e…` 被保真守卫拒绝，未启动镜像。只读提取原归档 config，重新确认原 SHA 后对比：两者均为 10,173 字节，整个 JSON 字段树、runtime config、history 与 rootfs diff IDs 完全相等。差异仅在序列化字节，不能把另一摘要误称为原身份。

保持失败布局不变，新建 OCI 外壳、复用原 layer blob，并引用原 config 字节；恢复 config 摘要到原 Docker ID，新的 OCI manifest 摘要单列为 `3c1b8b42…`。这不是重编译、换 JAR、字节码补丁或修改文件层。三个生成回归实际确认等义重序列化可以保留原字节，而字段改变或错误原始摘要必拒绝。回滚仅需不再使用新外壳：原下载归档、失败布局和原始 config 均保留，不操作用户工程/数据。

后续完整核验与 rootless 解包实际成功，190.548 秒、CPU 157.217 秒、内存峰值 768 MiB、swap 0；unit 限制 2 CPU、768 MiB、64 Tasks、仅 AF_UNIX。这些包含文件缓存的准备用量不是 Reader RSS、运行时预算或长期负载结果；未保留完整资源事件计数，不声称 `memory.events.max=0`。

- 14 个 OCI layer 的字节大小和 SHA 全部复算通过；原 config 字节摘要与 Docker ID 相符，转换后的 OCI manifest 身份另列。
- 镜像内 `/app/reader.jar` SHA 为共享托管 JAR 的 `1c1c24bd90a7d08f48ef016d1b2578650d4aba0574aa3b53662d5b1ea2bf302f`，User/入口/renderer/版本/合并快照声明全部核对。
- 已安装发行包元数据为 Camoufox 0.5.6、Playwright 1.62.0；安装的浏览器元数据为 152.0.4-beta.30，所记录资产 SHA 与固定锁相同。没有执行镜像内解释器或浏览器，不能由此宣称运行版本/渲染通过；本机也未另行下载原浏览器 zip 复算。
- rootless umoci 对 `usr/lib/x86_64-linux-gnu/gstreamer1.0/gstreamer-1.0/gst-ptp-helper` 的 `security.capability` xattr 返回 EPERM 并忽略。此警告保留；不宣称解包后的所有权限/扩展属性与 Docker runtime 完全相同。
- 新 Reader、浏览器、真实凭据、生产修改均为 0。准备 unit 已退出，下载进程已退出，用户 18931/PID 61244 仍监听，4056/18944 没有复活。

[完整脱敏身份与边界](evidence/local-image-unpack-2026-10-04.json)记录两种 config/manifest 摘要、实际检查和本机原报告 SHA。运行前仍需解决 `newuidmap/newgidmap` 缺口；没有用本机 root 或宿主 Chrome 绕过它。

## 非 root 用户的真实缺口

首次小探针漏掉 Ubuntu usr-merge 的路径，修正为 `usr/bin/busybox` 后再运行。单一 UID 10001 映射被 runc 明确拒绝：缺少 UID 0 映射；没有改成 root 用户测试以冒充镜像用户。

按本机现有 `/etc/subuid`、`/etc/subgid` 中已经存在的范围补齐两段映射，进程仍为 UID/GID 10001。第二份正确映射的空 OCI 探针实际失败于 `mapping tool not present`；它未运行 Reader、未读取真实数据。此处是缺 `newuidmap/newgidmap`，不是已证实的 Camoufox 或 Reader 失败。

已询问用户是否允许仅在 WSL Ubuntu 安装官方 `uidmap`。收到明确答复前，不执行安装、不修改系统映射、不开 Docker 服务，不降为 root 来绕过这个验证。完整目标保持 active；当前局部运行前置条件待授权。

## 制品与后续验收门槛

下载实际沿用同一次句柄，没有因一次观察超时重启；本机 `reader-image.tar.gz` 与托管元数据的归档 SHA-256 已相符。此处只确认下载字节，不替代后续 OCI、JAR、浏览器包/安装元数据及运行验收。

本轮不删除旧工程、测试报告或用户缓存。后续运行须验证镜像内 JAR `1c1c24bd90a7d08f48ef016d1b2578650d4aba0574aa3b53662d5b1ea2bf302f`、实际 UID 10001、回环监听、2 CPU/2 GiB/256 PID 与零 swap、凭据撤销和进程树回收。Docker 归档转 OCI 必须保留内容身份并清楚记录格式转换，不把不同 image ID 或替换 JAR 写成原字节传递。

真实起点仍需合法搜索/免费第一章以及原 JAR、历史远程、内置引擎的同条件对照；扫码/验证码与付费权限不能自动绕过。本机前置能力、下载或托管合成绿灯均不能替代这些完成条件。
