# 当前完整 Native 镜像的本机短测

## 结果与范围

2026-10-10 已完成当前 eb30／58fb 的完整 AMD64 镜像下载、归档及14层散列核对、rootless解包，并用该镜像原入口实际运行生成书源短测。原 JAR／只读8d依赖组合诊断不能替代这个结果；本轮也不借组合诊断冒称最新整镜像已执行。

实际 UID/GID10001，空附加组／空能力／禁止提权／仅loopback；宿主独立核对映射110000、六个private namespace、七个只读挂载及cgroup。实际原入口启动、发布身份、Vue3首页／静态资源字节、生成搜索／异步脚本／Cookie隔离与删除／延迟详情／并发4通过。**这是一次43.108秒的生成API短测，不是本机渲染UI、冷启动压力、长期或真实认证书源验收。** 本机未使用宿主Chrome或独立WebView容器，浏览器／Java／Python业务依赖来自这份完整镜像；runc等宿主工具仅用于诊断隔离。

独立读取原报告／执行源码并重新核对现有默认UI／异步／metadata／父级预算守卫，有限回执 SHA `86475562c989325284d52bad1c3a0ef63ea1a01ea5a1728abb223baed86a4d07`。第一次复验器误将CPU周期预期为字符串而失败；保留原复验器，第二版要求实际整数100000，不改业务数据、原报告或资源上限。

## 来源和实际散列

- 已独立接受的源码 `eb30cb6bfb935bc558d05c1c609293691da7b16a`，实际tested merge `58fb51b1091719b8de8954fb430e82c6cbcf3382`。四条托管流水线及25实际UI／443 Python／双clean／Native六实际作业等结果见[自己的托管记录](EXACT-JAR-INIT-AND-UTF8-2026-10-10.md#eb30-新增量自己的托管验收)；原件三方的严格UTF-8失败仍保留。
- 当前 Native run [38019762930](https://github.com/warpdotsys/reader-dev/actions/runs/38019762930)，独立回执 SHA `7ccd2b48b5129895f6e0485237bed54618d7d8cf53980616e8b77a6da770499c`。制品11658665042／2,174,866,338 B，源码、run、名称、未过期状态及声明digest在下载前重新核对。
- **完整镜像归档本机已复算**：`reader-image.tar.gz` 为2,174,859,811 B，SHA `9f5a2cbe4f5585f683e8b971a47bec793faba2c859803f2800299dc5e4d9b0aa`。GitHub ZIP声明digest `sha256:f60c20f12bd68824011fcbb48f95b58fc319677af16c4b513c3817b1c60b81ee` 未单独本机复算，不冒称ZIP验证通过。
- **原Docker config字节保留**：ID `sha256:601f65abd91cee4b8dcb59e8675b696c0284efe61948928849966c600a533468`。转换工具重新序列化曾产生 f12 config；仅在新的OCI wrapper内恢复原始config字节，先验证完整JSON语义严格一致，保留原转换布局，未修改层／Java类／脚本行为。新的OCI manifest为 `sha256:3a5b94137d21ba1f832af9f75d29ce5ca6283cbf1c57bce36a45a3d93f4a8c22`；14个完整OCI层blob逐个复算，并非声称额外手动复算所有uncompressed diff-ID。
- **JAR本机已复算**：仍为 `e59a01bf190de757f73216557ff353411019507d25bad76f0c808b48366adb84`，不是原件b26，也不是旧8d的591 JAR。镜像Python包元数据为Camoufox0.5.6／Playwright1.62.0，浏览器版本元数据与固定asset锁一致；浏览器原始asset ZIP没有再次单独本机复算。

独立目录 `D:\Codex-reader-artifacts\20261010-eb30cb6b-a`；新解包目录 `/home/chong/.cache/reader-ci-image-eb30cb6b-20261010-5gyybv0e`。原始JAR、以前的归档／布局／失败及用户dirty全部保留，不重复下载旧镜像或覆盖它们。

## 三个顺序阶段的真实资源

| 阶段 | 预算与实测 | 结果 |
| --- | --- | --- |
| Windows下载 | 32逻辑CPU的Job rate625／10000，约2逻辑CPU；提交内存上限536,870,912 B、峰值156,196,864 B，735.051秒 | 下载完成；不能把提交内存限制当包含宿主文件缓存的总内存 |
| 非root解包 | 独立父级2CPU／2GiB／high1.5GiB／64PID／零swap；峰值1,612,963,840 B／PID8，high20,375、max／OOM／PID触限0 | 实际归档／层／config／JAR核对通过；尚未启动Reader |
| 完整镜像生成短测 | 独立父级2CPU（0–1）／2GiB／high1.5GiB／256PID／零swap；峰值1,611,571,200 B／PID220，high2,881、max／OOM／PID触限0，43.108秒 | 实际API／静态字节及严格父级预算通过，清理剩余0 |

下载、解包和Reader运行顺序执行，不让两个本机重任务同时运行。解包与运行分别记录资源；它们不构成同一冷启动／长测样本，也不把缓存峰值当纯JVM RSS或无泄漏证明。提前回收频繁，性能／长期风险仍保留。开始运行前WSL实际MemAvailable约6,754 MiB；没有主动shutdown／terminate／重启WSL或全局drop缓存。

## 实际启动条件和清理

镜像原 `reader-entrypoint`、全部字体与浏览器功能及镜像JVM堆环境保留，不覆盖JAVA_OPTS或拿新heap让测试通过。诊断PID1只回收该private PID namespace的收养子进程；它不是镜像默认PID1拓扑，不能据此冒称原始Docker启动的所有生命周期行为都验完。

测试专用环境明确限定：端口18890、bind127.0.0.1、workDir `/`、secure=true、licenseCheckEnabled=false、浏览器超时5000ms、允许隔离loopback夹具的private network，以及禁止Python写字节码。这些设置不写回生产或项目默认值。挂载为只读镜像与只读验证脚本，storage/logs/home/cache为新生成tmpfs；没有挂载用户浏览器会话或真实storage/data。

实际主诊断退出0，回收56个收养子进程，Java被本测试有序SIGTERM停止、实际退出143；**不是Java自然退出0**。独立确认容器stopped/pid0、删除／状态清单剩余0，worker／browser／driver均0，两自有systemd单元实际inactive／MainPID0。日常Reader和生产服务未操作。

原件b26及58份受保护用户文件在独立复验中再次散列一致。没有复制新真实正文／Cookie、购买章节、改代理／已有UID映射、安装宿主Chrome／Docker或重写Go/Rust。

## 已知限制与回退

- 原件＋历史WebView的UTF-8 POST仍截断44／60 B，严格六例三方仍失败；最新完整镜像短测成功不覆盖这个结论。[准确三方边界](EXACT-JAR-INIT-AND-UTF8-2026-10-10.md)。
- 本轮只有生成书源；真实登录书源、完整请求头／Content-Type差分、真实正文及生产登录／阅读仍待验。25实际界面旅程和新截图来自GitHub托管runner，不能替代本机可见UI或生产验收。
- OPDS后端及独立账号配置仍未实现；截图通知隔离是测试证据修正，不是OPDS功能交付。
- rootless解包记录过gstreamer `gst-ptp-helper` 的 `security.capability` xattr被拒绝；不声称所有rootfs权限／xattr逐项完全相同。进程无能力且本轮生成用例通过，但该差异对全部媒体功能尚未验证。
- 新原型诊断／读取回执在本机独立目录，16个准备安全单测通过；不是已经并入正式生产启动器。后续正式版本仍须由已验源码经GitHub托管runner发布并部署验收。

回退只停用这些新诊断入口或精确自有单元（两单元已结束），保留新旧归档、原件、完整报告和失败，不改生产、不删除缓存／用户数据、不reset或覆盖既有工程。完整报告和执行副本仅在本机；[有限机器记录](evidence/local-current-native-2026-10-10.json)可公开，不包含正文／凭据。
