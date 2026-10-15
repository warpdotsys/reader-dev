# v6.1.0 本机完整镜像与持久数据验收

用户已指定：“连不上就用本机，后续都这样”。后续服务器连接失败默认继续本机，不重复索要 SSH 信息；生产未验收独立记录，不再阻挡本机交付。

## 实际可用入口

- Reader：<http://127.0.0.1:18963/>。
- 生成测试账号：`localfull610`；测试密码：`Local-Test-610`。仅供本机生成数据测试，不应用于生产。
- 数据与软件：`D:\Reader-Pro-Local-Full-610-20261015-a`。
- 控制脚本：`D:\Reader-Pro-Local-Full-610-20261015-a\runtime-software\source\ReaderLocal.ps1`。

使用 PowerShell 7：

```powershell
& 'D:\Reader-Pro-Local-Full-610-20261015-a\runtime-software\source\ReaderLocal.ps1' -Action Start
& 'D:\Reader-Pro-Local-Full-610-20261015-a\runtime-software\source\ReaderLocal.ps1' -Action Status
& 'D:\Reader-Pro-Local-Full-610-20261015-a\runtime-software\source\ReaderLocal.ps1' -Action Stop
```

停止会退出本次 Reader / 许可证程序并卸载本次写时复制层，保留数据盘和镜像盘；不停止 WSL、不改 WSL 全局设置、不删除用户数据。没有设置开机自启动。隐藏的 WSL 控制进程在运行期间保持本次服务，不能随意结束它。启动只重挂已有磁盘，绝不重新格式化。

旧 Windows 演示 `18961` 与来源 `18962` 保留，未替换或停止。它们与本次完整 Linux 镜像是不同实例。

## 身份与边界

这是正式 AMD64 镜像的准确文件系统与原入口，不是给旧镜像换 JAR：

- 应用提交 `cc4593a1f707b30366005974f45eb1436da9b8fb`。
- JAR SHA-256 `b4954ac70229f4514146b8e077909874bfc5aeee779f71fde903622638ed1ee0`。
- 原 Docker config `57403ef4e3d7412b049636985e4f3d2c950e3aaf94d917b21520c6c8309421d9`，14 层。
- 包内 Camoufox 0.5.6、Playwright 1.62.0、浏览器 152.0.4-beta.30；无需个人 Chrome 或独立 WebView 容器。
- rootful 解包，运行 UID/GID 10001、空 capabilities、no-new-privileges，完整根目录写时复制，原软件层不改。

许可证程序使用项目已有 Java 实现，重新编译及两项测试全部实际执行通过，fatJAR SHA-256 为 `7d348ecbb121b50ff76f96a8f84b1f4a01f1d74f3923026a55d533b138eb4d53`。它作为额外只读软件挂入独立的本机许可证进程容器；Reader 和许可证复用同一准确镜像的只读底层，各有独立 PID / mount 空间和写时复制层，合计仍受同一个 CPU 2 核 / 内存 2 GiB 父组限制。不能声称许可证服务原本已包含在正式镜像里；这也不是独立 WebView 容器，浏览器仍内置于 Reader。它在本机生成独立密钥，Reader 正常启用在线校验、信任对应公开密钥并完成签发/激活/验签，未禁用授权校验，未使用生产密钥。

服务使用 WSL host 网络，不是无外网 network namespace；Reader 和许可证程序都只绑定 `127.0.0.1`（18963 / 18964）。默认拒绝浏览器私网访问；仅生成回环书源验收时显式启用临时开关，测试后停止服务并以默认关闭配置重新启动。没有改代理、DNS、防火墙或公网端口。

## 首轮实测结果

1. 准备器 / PID 1 / 服务控制器的 26 项单测在本机 WSL 通过；许可证 2 项 JVM 测试通过。此处不是 GitHub Actions 的新一次结果。
2. 首轮 API：正确/错误密码、退出/重登；正常许可证激活；69,899 字节三章生成 EPUB 预览、保存、目录、三章正文与 UTF-8 检查通过。
3. Reader 包内浏览器实际执行生成网页的延时 DOM + Promise + fetch 标记，返回预期书名；临时书源已删除，fixture 已关闭。仅一次小场景，不替代此前完整并发与三方证据。
4. 停止后重新启动：许可证公开密钥、安装许可证、生成 EPUB 的哈希一致，账号登录、唯一生成书架和已激活授权保留；默认私网测试开关恢复关闭。
5. 再次立即停止/启动成功。实际页面登录后打开第 3 章、点击目录第 1 章、刷新，以及第二次服务重启后刷新：页面标题、正文标题与全部 64 个段落编号均属于第 1 章，无 U+FFFD 替换字符。
6. 首轮运行组内存峰值 1,312,886,784 B；CPU 硬上限 2 核、内存硬上限 2 GiB / high 1.5 GiB / swap 0，OOM 与硬上限事件为 0。不是整个 Windows 主机或旧演示的总内存。

实际浏览器里有第三方扩展脚本异常，另有未定位来源的 MutationObserver 异常。本轮不声称“控制台零错误”，也不未经授权停用用户扩展；上述中文与章节验收以真实渲染结果为准。

## 追加的启动器隔离验证

审查发现首轮同 UID、同文件空间的两个 Java 程序虽未共享环境变量，Reader 仍能直接访问许可证令牌和签名数据。现已拆为两个独立受限角色：Reader 仅挂载书库与自己的管理密钥，许可证角色仅挂载签名数据与自己的令牌；日志目录也分离。两者均为 UID/GID 10001、空 capabilities、no-new-privileges。没有换 JAR、重签许可证或改数据格式。

实际在两个运行中的角色内部执行检查后，`isolation-acceptance.json` 已接受：交叉密钥不可见、对方数据目录未挂载、PID/mount namespace 不同；原生成 EPUB、已安装许可证和公开密钥哈希均未变，默认私网拒绝仍启用。检测只读取进程元数据与布尔状态，不读取密钥内容或私人正文。

源码更新现在必须取得控制锁、确认服务停止、校验全部旧文件，再保留旧源码快照、逐文件写临时副本并原子替换，最后更新锁清单。启动时核对全部七个持久化软件文件及其来源，拒绝混合或不匹配的源码。首次在 D 盘执行时遇到 DrvFS 打开的文件句柄延迟发布替换结果；当时仅已备份旧清单，未替换运行源码。改为 fsync 后先关闭输出句柄再替换，补回归测试后实际更新、启动成功。原失败历史目录保留。

## 生成命名空间恢复与最终界面

已实际新建一个随机 `rollback610*` 测试账号，通过 Reader 导入三章纯生成 EPUB。停止服务后仅备份它自己的 `storage/data/<username>`：16 个文件、333,368 字节，清单包括每个文件的 SHA-256、大小和文件/目录 UID、GID、权限。

重新启动后通过真实 API 删除该测试书，确认书架已无此书。再次停止服务，先从快照构造并完整验证临时恢复目录，再将删书后的目录改名到测试专用 quarantine，最后原子切换恢复目录。旧目录和快照均保留；没有读取其他 namespace、复制整盘或覆盖全局用户注册表。恢复后实际重登、书架、三章正文标题与 UTF-8、许可证 `verified=true` 全部通过。全局账号注册表和许可证私钥不是本次备份对象，因此这不是整站灾难恢复证明。

隔离角色中的内置浏览器另行完成一次真实延时 DOM / Promise / fetch 生成网页测试，测试书源已删除、fixture 已关闭。随后按默认私网拒绝配置重新启动，原 `localfull610` 阅读页刷新成功：标题和全部 64 个生成段落归属第一章、无 U+FFFD。实际截图保存在 `ui-isolated-restart.png`。包含浏览器请求的运行组峰值 1,435,127,808 B，high/max/OOM 事件均为 0，干净退出码为 0。

本次演练的原失败也保留：预览临时 URL 的目录请求被拒，改为先 `saveBook` 再使用返回地址；已创建的随机账号通过本次明确身份的修复脚本续接，没有新建其他账号。停止后 systemd 回收临时 unit 时会返回 `inactive`/退出码 4，现另行验证 `not-found/inactive/dead/MainPID=0` 才接受；WSL 自然休眠后只重挂已核验的数据盘，绝不格式化。演练器默认只给计划，真实执行必须显式 `--execute`；发现不明 pending、软链接、跨 namespace URL 或哈希不符时拒绝继续。

本机 WSL `local_reader_*_test.py` 45 项通过（含真实运行发现的回归），流水线安全守卫 128 项通过。一次本机全 Python 探测未通过：869 项中 4 失败/7 错误，其中 10 项依赖本机 WSL 缺失的 Node，另一项假定非 root 身份；该次失败不是通过记录。完整 Python/前端/JVM 与双 clean JAR 比较交给现有 GitHub 托管 Ubuntu CI，新增测试自动参与 discovery，并保留 Python 成败日志。以对应提交的 [Java Kotlin CI](https://github.com/warpdotsys/reader-dev/actions/workflows/ci.yml) 结果为准，不把本机单测等同于托管全套或让托管 runner 执行本机磁盘操作。

## 失败记录，不覆盖

- 首次生成 EPUB 验证器错误使用了预览临时地址；`saveBook` 实际返回持久地址。验证器改为读取保存响应后重跑通过，没有修改产品。
- 首次快速重启在控制器中报 OSError；日志未带 errno，因此未断言唯一根因。端口预检增加 SO_REUSEADDR 以允许 TIME_WAIT、仍拒绝活跃监听，并补充有限 errno 诊断。之后再次立即停止/启动通过。旧控制器日志与旧源码快照保留。
- 旧许可证离线构建缺依赖缓存；补到 D 盘后用 `--rerun-tasks` 实际编译、执行测试和打包成功。未把 UP-TO-DATE 当作新一次测试执行。

## 文件、空间与证据

`image.ext4` 逻辑 16 GiB、`data.ext4` 逻辑 4 GiB，均为 Windows 稀疏文件，实际占用按写入增长。镜像转换的 `/tmp` 和 `/var/tmp` 实际绑定到 D 盘文件系统；不只是设置 TMPDIR。正常运行的小启动软件包为 28,293,330 字节（初次快照），已固定到 D 盘，不依赖可清理的仓库 build 缓存；仍依赖当前 Ubuntu WSL 的 systemd、glibc、mount / loop 能力，并非跨发行版免依赖安装器。

验收回执均位于上述 D 盘目录：

- `prepare-receipt.json`：归档、层、原 config、JAR、权限、浏览器元数据。
- `runtime-software/runtime-software.lock.json`：小运行包逐文件哈希；更新前源码保存在 `source-history`。
- `verification-initial.json`：生成账号、许可证、三章 EPUB、实际嵌入式浏览器。
- `restart-before.json` / `restart-after.json`：重启持久化；`service-first-stop.json` / `service-second-stop.json`：资源峰值和有界退出。
- `ui-reading-check.json` / `ui-final-check.json`、`ui-final.png`：页面登录、切章、刷新及第二次服务重启后的正文就绪检查。
- `isolation-acceptance.json`：独立角色实际权限和命名空间否定检查、原数据及授权身份保留。
- `rollback-{seed,snapshot,damage,restore,verify}.json` 与 `rollback-seed-attempt-1.json`：生成恢复演练及原失败；不含凭据或正文。
- `browser-isolated-roles.json` / `service-rollback-stop.json`：新隔离角色实际浏览器请求、峰值与退出；`ui-isolated-restart.json` / `.png`：最终实际界面。
- `service-state.json`：当前启动实例；不是不可变历史证据。

未读私人正文、未接入个人 Chrome/起点会话、未导入生产数据。原 JAR、只读备份、获准单本 EPUB 和 58 个保护报告已仅以哈希核对不变；Git 用户 index 未变。

## 尚未验证

本轮没有证明公网生产部署、真实生产账号、起点认证、全部书源/全部藏书；没有做机器重启或 Windows 注销后的自动恢复，也没有做整盘/全局账号表的灾难恢复。已完成的是单个纯生成 namespace 的实际恢复，不以重复远程连接代替本机交付，也不扩大结论。

子进程应用日志有轮转上限；外层 `controller-*.log` 则保留历次小型启动/失败记录，目前不会自动清理。这一维护限制与未启用开机自启动均保留，不将本机工具称为通用安装器。
