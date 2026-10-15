# 前端依赖安全补丁候选

## 已从官方公告核验

本轮生产依赖审计原有五个 high 包条目，不等于五种全部可达的攻击。只更新下列修复族，不引入全量重写或后台存储迁移：

| 依赖 | 原锁定 | 新锁定 | 修复依据 |
| --- | --- | --- | --- |
| Axios | 1.19.0 | 1.20.0 | [请求选项原型污染修复](https://github.com/advisories/GHSA-vh66-26gq-q6x8)及同版本相关公告 |
| Vue 及同版 @vue 族 | 3.5.40 | 3.5.42 | [SSR 属性名缺失回车过滤](https://github.com/advisories/GHSA-g2v6-rqmx-r4w6) |
| nanoid | 3.3.16 | 3.3.18 | [零尺寸生成器无限循环](https://github.com/advisories/GHSA-2v37-7h3g-55p8) |
| source-map-js | 1.2.1 | 1.2.2 | [索引映射偏移放大](https://github.com/advisories/GHSA-68fv-2mgg-jv7q) |

Vue/Axios 直接依赖使用固定补丁版本；两个传递依赖在既有父范围内更新锁，不添加强制 override。锁生成工具顺带刷新的 Babel/parser、sourcemap-codec 已还原，Linux libc 元数据保留。静态对照确认只有目标族记录改变，未增加或删除其他包。

## 已验证与尚未验证

已对新锁执行只读 `npm audit --package-lock-only --omit=dev --json`：生产漏洞条目为零。语法检查及仅目标族改动检查通过；原 JAR、获准单书和 58 份用户报告哈希不变。该审计只反映当时已登记的依赖公告，不证明全系统没有漏洞，也不代替真实请求、登录、Cookie、阅读或完整镜像验收。

新增十项回归检查锁/实际安装版本，并执行真实库：零尺寸 nanoid 不调用随机源；源码映射拒绝超大、非法和嵌套累计偏移；Vue SSR 不把回车属性名变成事件/自动聚焦；Axios fetch 在仅回环的生成 HTTP 请求里保留显式方法与生成鉴权头，不采纳污染头。危险样本使用独立子进程、128 MiB JS 堆和五秒期限，超时必须失败，不能把没有崩溃当成修复。测试不导入用户密码、Cookie、真实账号、正文或浏览器会话。

独立最小样本最终十/十通过，测试与正式 helper 源码完整字节一致，不是 stub 或伪实现。首轮三个通过/两个失败，纠正污染描述符不可写和索引映射列边界样本后，仍有真实 Axios 反例：1.20.0 的已解析 Request 包含正确头，但第二参数缺少自有 headers/method，Node 24.19 的 fetch 最终采用原型继承的头。没有删除原断言或宣称仅升级版本就修好。

因此新增可读 `readerFetch`，只接入 Reader 中央 Axios 实例的 env.fetch：保留已解析 Request 的方法、实际头、body、Cookie 策略、取消信号及其他传输值为显式自有字段；不改变默认 XHR 选择或原 15 秒期限。Axios 显式 manual 重定向仍优先。回归另覆盖真实 JSON/FormData、取消和禁止跟随重定向，以及输入/冻结选项不变和错误原样传播。Node FormData 默认 MIME 与守卫无关，两边对照同为 urlencoded；测试采用显式 multipart 声明核对 native 边界，没有把错误 MIME 当产品守卫修复。真实浏览器业务仍须自己的新托管流程复验。

本机宿主物理可用内存不足，不启动 WSL、Firefox 或完整 Gradle 构建；当前旧 node_modules 也不能冒称已安装新依赖。只建立独立最小依赖样本核对新回归，整前端、类型检查、双 clean、实际业务和单镜像均交给本提交自己的 GitHub 托管 runner。不得沿用 b2ed 的绿灯证明这一补丁。

[独立真实库样本、未接受反例及源码摘要](evidence/frontend-security-deps-local-2026-10-13.json)。本轮只证明这些回归样本和 Reader 中央 Axios 的防御范围，不宣称所有原生 fetch 或其他环境均可抵抗任意原型污染。源码守卫不执行用户脚本，不清理用户浏览器会话。

## 本候选自己的四套托管验收

源码 `548a387949a40b1ce06fac0ba9d44c3d6c39c65a`，受测合并快照 `87b0e768951ff0e4e063a095d50138a36aefab04`；不是复用 b2ed 的结果。四条流程均已终态成功，小报告下载后由独立消费者重新检查身份与业务断言：

- [Java/Kotlin 38354402452](https://github.com/warpdotsys/reader-dev/actions/runs/38354402452)：47 套件、222 项、35 项既有环境跳过、零失败/错误；跳过不计通过。两次 clean 的完整 JAR 字节相同，SHA `16f432e5341da9f89a55d82d10b383cd25c5c4cd74b82eebb3fa797071fc6e5a`，285,683,521 B。
- [Vue 38354402228](https://github.com/warpdotsys/reader-dev/actions/runs/38354402228)：新依赖冷安装后 376 前端测试、类型检查及 Vite 构建通过；25 个实际 Chromium 旅程无失败/跳过。
- [完整镜像 38354402266](https://github.com/warpdotsys/reader-dev/actions/runs/38354402266)：755 Linux Python 测试无跳过，Reader/worker 的生成 HTTPS、脚本/异步和资源报告复验；完整镜像默认 UI 旅程 13.064 秒。当前生成登录、阅读两图实际查看，中文可读、不空白，不推断所有页面或真实账号都通过。
- [双原生架构 38354402243](https://github.com/warpdotsys/reader-dev/actions/runs/38354402243)：共享 JAR、AMD64/ARM64 原生构建、各自重导入和正式发布器导入共六个 GitHub 托管作业全部成功。24 个真实默认引擎契约无跳过；32 份发布器 JSON、两个架构 metadata 和重载身份独立一致。未推镜像仓库或正式发布。

[有限公开回执](evidence/frontend-security-deps-hosted-2026-10-13.json)明确分开短测、完整制品和本机状态。普通短测允许 1 GiB swap 而实际为零，不冒称零 swap 长测；UI 客户端在服务端预算之外，不声称合计 2 GiB。Full/Native 服务端子集内存峰值为 886,145,024/899,850,240 B，PID 峰值 207/199，不证明长期无泄漏或真实大书容量。

当前准确归档的三项专项已分别终态成功，并下载报告独立接受：

- [AMD64 30 分钟离线长测 38357513408](https://github.com/warpdotsys/reader-dev/actions/runs/38357513408)：1807.259 秒/128 轮/520 次成功搜索。
- [ARM64 30 分钟离线长测 38357516041](https://github.com/warpdotsys/reader-dev/actions/runs/38357516041)：1809.375 秒/112 轮/456 次成功搜索。
- [固定公开 Gutenberg 网络书实际 UI 38357513168](https://github.com/warpdotsys/reader-dev/actions/runs/38357513168)：已终态成功且完整报告独立接受，具体范围见下节；不是网站真实认证三方。

两架构长测均消费 Native 38354402243 的原归档，不重建或换入新 JAR；保持 2 CPU/2 GiB 硬上限、1.5 GiB 早回收、零 swap/256 PID 和原断言。[原 ZIP/全部解压字节、每轮轨迹及终态清理的独立回执](evidence/frontend-security-current-soaks-2026-10-13.json)均通过；内存峰值 AMD64 1,001,299,968 B/ARM64 976,560,128 B，PID 峰值210/203，硬上限/OOM/PID触限与实际 swap均零。各轮空闲浏览器/worker/driver归零及三种故障后的恢复通过；空闲内存较首轮仍增长70,156,288/63,283,200 B，不能宣称没有泄漏或生产容量全部达标。

## 准确最新镜像的公开网络书与真实 UI

专项 `38357513168` 的探针源码为 548a，实际加载的原生 AMD64 镜像是已接受 Native 38354402243 的受测 87b 快照，image ID `sha256:de1f40226f5946cd56bc4bf04b0c030ba9dd933ce393024771e88bfa133b061f`、JAR 为同一准确 16f。执行期间未重建、换 JAR、mock 响应或依赖宿主 Chrome；测试客户端使用同镜像的打包浏览器，不是部署依赖的独立 WebView 服务。

实际 Reader 默认 Vue3 UI 注册、登录、入架、阅读 0→1→刷新 1→返回 0 并退出，29.428 秒。28 目录、四次画面 6,822/7,332/7,332/6,822 字符与实际 API 仅空白归一后完全相同，替换字符、页面错误、被阻断客户端请求和接口观察错误均零。生成命名空间、会话撤销、两自有测试容器移除及实际 Reader/API 业务均由原守卫核对；不是只检查 HTTP200 或截图存在。

重新取得原 64,850 B ZIP，SHA `06557a96b3aa7255c9e79c50839d925fa6e2f1d7d48c988851657f7448933c41` 与 GitHub artifact digest 一致；全部 15 个解压文件字节与归档逐项相同。755 Linux 验证测试无跳过；原 UI/API、身份、资源和清理守卫保持。登录/退出两图通过像素护栏并实际查看，中文可读、不空白；正文画面没有导出，真实凭据/本地私人书籍读取为零。

[有限公开证据](evidence/frontend-security-current-public-ui-2026-10-13.json)保留独立原回执 SHA、准确制品和原归档文件 SHA。Reader 服务端与外部 UI 测试客户端各自守住 2 CPU/2 GiB/256 PID/零 swap，**不是合计 2 GiB 通过**。这轮只证明固定匿名 Gutenberg 网络书和 Reader 生成登录，不证明起点认证、原件/历史/内置引擎实站三方、获准私人书复测或生产上线；两架构30分钟长测为另一次无外网生成数据实测，不拼接为真实书源30分钟接受。

## 后续准确原件对照的只读准备

先前用户批准的服务器隔离根仍存在，其中原始只读副本重新核对为准确 `b26fb476...`；既有恢复副本仍是旧 `221d41ef...`，不能当本轮 16f。只读查询也确认既有旧 190e 非 root 运行环境与固定历史 renderer 可被本机 Docker 识别；这不证明新 Native 镜像已在服务器装载，不将旧运行时身份借给新产物。

第一轮本机单 JAR 下载预检因物理内存不足终止，未创建目录或传输；该失败保留。后续宿主余量实际满足同一门槛，独立 Windows Job 的约2逻辑CPU/512 MiB提交内存/64进程约束下，91.695秒下载，实测峰143,106,048 B；准确16f/285,683,521 B与双clean及两Native metadata相同，全部1,568条目逐项和旧8f比较，共1,534同名项，非Vue内容/名称差异及class变化均零，Worker仍17c47。[实际重哈希及完整条目回执](evidence/frontend-security-current-jar-2026-10-13.json)。它不证明原始B26完全兼容，也不等于最新完整Native已在本机执行。

仅将这份公开软件副本和诊断代码放入原批准服务器隔离根的新子目录，106.592秒、实测本机Job峰89,722,880 B，远端SHA相同，输入移除写权限；原件不重传，旧输入不覆盖，无真实正文或凭据。最新准确JAR的生成三方与UTF-8观察已完成，详见[实际结果、失败保留和边界](CURRENT-ARTIFACT-THREE-WAY-2026-10-13.md)。镜像承载环境仍是旧固定190e，不能代替最新完整Native/获准私人书或真实站点认证。Goal继续active，未合并、正式发布或部署。

## 重建与回滚

项目 `web-vue3` 中运行 `npm ci`、`npm test`、`npm run build`、`npm run audit:security`；再用 JDK 11 执行项目的 `gradlew.bat -PreaderWebUi=vue3 bootJar --max-workers=2 --no-daemon`。审计需要 npm 官方服务连通，失败不自动放过或改源。类型/接口/原版排版和双架构产物必须以新源码及实际报告复验。

回滚只恢复本轮 package/lock 的依赖记录和新增安全回归/审计脚本，然后重新冷安装和构建；不得恢复或覆盖用户 reports、storage、原始 JAR或 Go/Rust/旧项目。b2ed 的源码和准确 8f2e JAR 作为此前对照保留。两轮 529b 大书八/八仅属于旧依赖组合，不能移借为新依赖已通过；准确产物本机、剩余三方/实站/长时仍为缺口，未合并、发版、部署，Goal 未完成。
