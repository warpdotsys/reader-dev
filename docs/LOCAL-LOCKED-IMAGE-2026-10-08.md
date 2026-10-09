# 本机原字节镜像准备与未完成的完整启动

## 已验证的准备

本次只消费 [Native37730974808](https://github.com/warpdotsys/reader-dev/actions/runs/37730974808) 的一个原 AMD64 归档：业务源码 `8d0d03c0…`、镜像合并快照 `3fe3777a…`、JAR `591ff…dcec`、image ID `sha256:401d895c…44`。**不是 47 新镜像，也不是原始 3.2.14 JAR。** 原件、旧工程、日常 Reader 和生产均未修改；未读取真实书籍正文或导入凭据。

- 官方 `uidmap` 已安装，当前版本 `1:4.17.4-2ubuntu3`；未安装 Docker、Podman 或宿主 Java。`/etc/subuid` 与 `/etc/subgid` 散列仍各为 `d796e52b…be9cbd`，现有映射范围未改变。
- 收到“仅安装官方uidmap”的授权后，于2026-10-08 15:19 UTC只读复核：已装版本和官方 `archive.ubuntu.com/ubuntu resolute/main` 候选版本同为上述版本，无需重复安装；`newuidmap`／`newgidmap` 位于 `/usr/bin`，权限均为官方包的4755／root:root，`dpkg -V uidmap` 无校验差异。两映射文件完整SHA仍为 `d796e52bc335df4e55114fad949f19850e6b4008cf07bf8d53a4e88936be9cbd`。PID92449仍UID1000／D／`rtnl_lock`，且属于先前自有作业的 `exact-image` cgroup；没有重复启动运行器、修改映射或擅自重启Ubuntu。依赖已就绪不代表完整镜像已启动。
- WSL 首次下载在取得大文件前因网络不可达失败；没有改变代理或全局网络。之后使用已有 Windows GitHub 客户端，在本次自有进程树的 2 个逻辑 CPU 上限／512 MiB 提交内存限额内下载成功，720.618 秒、峰值提交内存 145,772,544 B（约 139.0 MiB）。这是 Windows 提交内存观测，不是 Linux cgroup 或包含文件缓存的总内存。
- 一份 `reader-image.tar.gz` 为 2,174,783,444 B，完整流式 SHA-256 实测 `e244742f72f037acf45eb0c2bf3d08edea9fe0bfb1bd7b03c0a25c9e40333518`，七字段 metadata 与锁定来源严格相同。下载的是 Reader／浏览器／依赖镜像，不是书库，也没有重新压缩用户数据。GitHub 客户端的临时 ZIP 已自行清理；**本机未复算外层 ZIP 散列**。
- 第一轮转换停在准备脚本错误的前端字段预期；真实镜像及 Dockerfile 均为 `READER_APP_WEBUI=vue3`，修正后 13 项生成校验测试通过，仍拒绝缺失、错误或重复配置，没有删掉 UI 门禁取绿。
- 第二轮复用完整转换目录，不重新下载或重做转换。转换只改变配置 JSON 序列化；逐字段语义相同后，在独立 OCI 包装目录恢复原 Docker config 字节／image ID，原目录、14 层和全部文件系统层保留。OCI manifest 包装散列与原 Docker image ID 分别记录，不混同。
- 离线解包实际成功，64.877 秒、CPU 39.251 秒、768 MiB 上限／峰值、零 swap。没有采集这次准备的 `memory.events`，**不宣称准备触限为 0**。根文件系统 JAR 完整散列、Camoufox `0.5.6`、Playwright `1.62.0` 及 `152.0.4-beta.30` 浏览器安装元数据与锁文件一致；未单独重算已安装浏览器 ZIP。rootless 解包不等于 UID 10001 运行。

原解包回执 SHA-256 为 `2c0500c198002ceeb6980ae7115cae7c0215e171a521eb315b28dc50ab0aaa97`，保留于忽略的 `build/local-camoufox-image-20261008-a/evidence/local-image-unpack.json`。准备源码和失败记录也只在该隔离目录，不打包进产品。

## 完整启动未通过：宿主等待与验收脚本错误分开

尝试按原镜像用户／环境／入口启动完整 Reader：只读原根文件系统、新生成存储、独立仅 loopback 网络空间；资源原定 2 CPU／2 GiB／256 PID／零 swap。启动前必须先取得运行身份并由宿主独立核对 PID／cgroup／网络，再写启动许可，之后才调用原 `reader-entrypoint`。

首次运行未取得身份握手。结束等待时，验收脚本的未捕获 `TimeoutExpired` 使正常报告未落盘；已保留该真实失败，后来补录的观察文件明确标注为观察记录，**不伪装成原程序当时生成的回执**。修正后的结束处理通过 6 项生成进程测试，等待失败不会再阻止保存结果；修正路径尚未做新的完整实跑。

运行器早期子进程 PID 92449 当时 UID 1000、D 状态、等待 `rtnl_lock`；它不是 Reader Java。启动许可文件仍为 0 B，cgroup 只剩该子进程，证明尚未允许原入口执行。该受限作业峰值 59,785,216 B、CPU 0.601 秒、swap 0；晚些资源读数内存约 26.8 MiB、PID 1、内存 max／OOM 0，不等于已跑过浏览器负载。

独立查询显示自有状态为 stopped／pid 0，但删除因 cgroup busy 失败，删除后清单仍有一项状态、内核子进程未证明结束；**不能说清理已成功**。其他已有进程也出现网络相关 D 等待，支持“当前宿主环境无法完成本次隔离初始化”的判断，不足以唯一确定内核等待的成因，也不是 Reader 业务启动失败的证明。没有重启 WSL、终止其他服务或反复叠加运行器。

失败观察文件 SHA-256 为 `1aea37ef9f2a04778b38cfea9e202e6228db9bfa84ac7fbcff17a3f85556b877`，保留于 `build/local-camoufox-image-20261008-a/evidence/local-full-reader-20261008-a-observed-failure.json`。当前本机完整服务、渲染界面、真实认证及最终完整三方仍未验证。后续若需重启 Ubuntu 以恢复环境，必须单独取得用户授权，因为它会中断已有服务。

## 其他验收与回退

47 自身的四条托管流水线／Native 六作业和复用 8d 原镜像的新增删除后专项均已独立接受，见[持续验收记录](BROWSER-SOAK-2026-10-08.md#新清理路径的实际托管运行)。它们不抵消本机未启动、匿名起点详情为空、真实认证和新最终三方待办。

这次没有业务实现或镜像热补丁；准备脚本只创建独立包装与可写测试挂载。停止消费测试目录即可退回既有流程，不删除原 JAR、旧根文件系统或失败材料。清理仅针对本次自有状态；内核 D 等待未解除时不强行宣称成功、不重复创建隔离进程、不擅自重启宿主。
