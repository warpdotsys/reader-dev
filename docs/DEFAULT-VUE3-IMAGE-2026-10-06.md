# 完整镜像默认 Vue 3 的候选增量

这是基于 `f4f47c70` 的新产品修改，不借用此前 `08cccfca`／`d5c00919...` 的镜像绿灯或长测验收。本轮未发 tag、写 registry、合并或改生产。

## 已从当前源码核实／已实现

- 普通 JAR 的 `AppConfig.webUi=vue2` 不变，包内原版 `web/` 保留；`-PreaderWebUi=vue3` 额外包含新界面。
- 完整 Docker 镜像新增 `READER_APP_WEBUI=vue3`，浏览器仍是镜像内的 Camoufox。Compose 用 `${READER_APP_WEBUI:-vue3}`，保留明确的旧界面选择，不迁移存储或更改账号。
- 镜像原生构建、传输后重载及发布导入必须验证默认 UI 选择；缺失、重复或 Vue 2 默认配置均拒绝。HTTP 探针不传 UI 选择变量，避免用测试配置掩盖镜像默认值。
- 新 `verify-reader-default-ui.py` 只访问显式 loopback 端口，禁代理／重定向，单响应最多 8 MiB、总计最多 16 MiB。实际读取首页、`/login`、`/search`、`/reader/generated-book` 的 HTML5 入口，要求 UTF-8／HTTP 200，逐字节匹配当前共享 JAR 的 Vue 3 index；入口 JS／CSS 同样核对 MIME 与包内字节，原版入口必须不同。
- 小报告含 JAR SHA-256、四条入口及静态资源散列，不包含 HTML、账号、Cookie、正文或动态错误。构建／重载保存 `DEFAULT_UI.json`，两侧发布导入再次按共享 JAR 散列严格验证，不能只保留首页状态码或静态布尔断言。

## 已成功重建／本机测试

前端类型检查和生产构建通过；`bootJar --offline --max-workers=2 --no-daemon -PreaderWebUi=vue3` 成功。新本机 JAR SHA-256 为 `1986ce336f9c9cc8012a5d6bb8c5618ac42b519545491b1dcaa6a62898faf0ff`，包内 worker 与当前源码一致，81 个 Vue 3 文件逐字节匹配本次 dist，原版入口保留。

新增 HTTP 生成夹具／负例 9 项通过，覆盖旧首页、深链 404、乱码、资源差异／错误 MIME、重定向禁止、非隔离地址、缺失／重复／篡改小报告。Vue 静态测试 239 项通过；Python 共 225 项，其中 224 项通过、Windows 的 POSIX 符号链接夹具 1 项实际跳过，不冒充 Linux 覆盖。发布链／原生身份／manifest 共 69 项通过，三个变更 YAML 解析及两个 Bash 脚本语法通过；不把它们称为镜像／渲染 UI 通过。

本机受限 Reader 首轮在新 JAR 身份确认前失败，次轮只有 UID 握手、没有结构化 Reader 结果；源码核查发现探针验证脚本挂载会被后续 `/tmp` 隐藏，已修正。第三轮旧运行时搭载新 JAR 实际启动自有 Java，但没有成功就绪／页面／回退观测，仍失败，具体产品原因未唯一定位。三次红灯保留，不造通过报告。第三轮 UID 10001／无外网／2 CPU／2 GiB／零 swap，原预算守卫通过，峰值 427,876,352 B／PID 44，触限 0；自有 Java 句柄停止、OCI 状态消失、user-systemd unit inactive／dead。没有运行原 JAR、读正文或使用真实会话。[本机重建、失败、有限观测与清理](evidence/default-vue3-image-local-2026-10-06.json)。它不能替代真正完整镜像的双架构门禁。

后续第四轮校验了新复制 JAR 的散列，但生成副本继承 `0600`，UID 10001 无法读取，Java 退出 1。只将该新生成副本改为 `0644` 后，第五轮实际通过：同一 `1986ce33...` JAR 的 Vue 3 四入口及资源逐字节匹配；切为 Vue 2 后首页也与保留资源一致，两个启动的业务就绪观测成功。保持无外网、非 root、2 CPU／2 GiB／PID 256、swap 禁用，42.789 秒，峰值 912,490,496 B／PID 67，触限／OOM 0。原守卫与回退结果已独立重查，自有句柄停止、OCI 状态消失、user-systemd not-found／inactive／dead。旧运行时只读未改；它的补充组、HOME 和挂载布局不完全等于 Docker 镜像，不冒充最终镜像或真实浏览器点击。

## 本修改自己的托管验收（2026-10-07 独立核对）

源码 `de7c0c7e`、受测 PR 合并快照 `bed59dc8...` 的四条 workflow 均已完成：

- [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/37484590395)。
- [Vue 3 浏览器旅程](https://github.com/warpdotsys/reader-dev/actions/runs/37484590435)。
- [完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/37484590411)的普通模式通过；其他条件模式跳过不计作真实站点或长测。
- [原生双架构构建／重载／发布导入](https://github.com/warpdotsys/reader-dev/actions/runs/37484590448)的六作业均成功。

下载的是小报告和生成数据截图，没有为此下载大型镜像归档或 JAR。24 份 publisher JSON 已检查精确文件集合、严格 JSON、两架构共享 JAR、四组实际 JAR／版本／镜像身份；5 份默认入口、5 份异步 GET／POST／脚本单次执行／Cookie 删除、5 份资源报告通过各自原守卫。普通烟测实际 swap 0，但配置允许 1 GiB，不能写成禁止 swap。两份默认 Camoufox XML 各 18 项、两份文档观测 helper 各 3 项通过；helper 不是 Reader 三方等价。Java 套件共 161 项，132 通过、29 门控跳过，另行实际引擎门禁和 23 项 Chromium 契约均无跳过。Vue 核心 18 项及子路径 1 项无失败／错误／跳过，登录 14 组几何已独立核对。登录和生成阅读截图中文正常。

原生发布链共享 JAR 为 `7848cc5e...`，完整镜像普通 workflow 自行构建的 JAR 为 `656d3fb4...`：各自报告按各自制品绑定，两者并非同字节，不宣称跨 workflow 字节可重复。大归档散列由 CI 校验并记录，本机没有重新下载校验。镜像 ID、资源峰值、报告清单散列和 artifact 编号见[自身托管证据](evidence/default-vue3-image-hosted-de7c0c7e-2026-10-07.json)。本快照不验收后续手机布局修复。

## 尚未验证与已知故障

本修改的普通镜像和 Vue 3 生成浏览器旅程已按上节独立验收，但静态 HTTP／字节检查仍不等于真实账号、全部页面、手机触摸或生产验收。托管书架悬浮截图曾在自动滚动后捕获标题经过半透明 sticky 导航的状态；本机新页首几何已排除“初始导航遮挡”猜测，随后实际复现手机分组标签与管理按钮重叠。这是此前 18 项绿灯未覆盖的真缺陷，修复及新断言必须单独验证，不能倒改本快照为手机无 bug。

此前 `08cccfca` 同镜像的起点匿名解析仍失败，本次不改浏览器业务源码，也不宣称修好认证／正文或原 JAR／历史远程服务的同输入三方兼容。静止内存增长与突发延迟的历史观察见[原制品长测](BROWSER-SOAK-2026-10-06.md)，不追认为新制品持续验证。生产仍未切换。

## 回退方法

后续[手机书架分组控件修复](SHELF-MOBILE-CONTROLS-2026-10-07.md)有自己的失败／通过 JAR 和新几何范围；本文 `de7c0c7e` 的镜像验收不覆盖它。

完整候选镜像可设置 `READER_APP_WEBUI=vue2` 选择保留的原版资源，无需重建或存储迁移。Compose 同名环境变量／`.env` 也是明确选择；当前源码不自动修改服务器上的 `.env`。若要撤销产品默认变更，撤回 Docker ENV／Compose 选择及相关默认 UI 门禁，保留本轮失败与实际报告。切换生产仍须按正式发布流程进行备份、实际登录／阅读验收与镜像回滚，不把静态入口检查当作发布许可。
