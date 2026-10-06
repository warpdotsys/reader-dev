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

新增 HTTP 生成夹具／负例 9 项通过，覆盖旧首页、深链 404、乱码、资源差异／错误 MIME、重定向禁止、非隔离地址、缺失／重复／篡改小报告。Vue 静态测试 239 项通过；Python 全套 225 项通过，其中 Windows 的 POSIX 符号链接夹具 1 项实际跳过，不冒充 Linux 覆盖。发布链／原生身份／manifest 共 69 项通过，三个变更 YAML 解析及两个 Bash 脚本语法通过；不把它们称为镜像／渲染 UI 通过。

本机受限 Reader 首轮在新 JAR 身份确认前失败，次轮只有 UID 握手、没有结构化 Reader 结果；源码核查发现探针验证脚本挂载会被后续 `/tmp` 隐藏，已修正。第三轮旧运行时搭载新 JAR 实际启动自有 Java，但没有成功就绪／页面／回退观测，仍失败，具体产品原因未唯一定位。三次红灯保留，不造通过报告。第三轮 UID 10001／无外网／2 CPU／2 GiB／零 swap，原预算守卫通过，峰值 427,876,352 B／PID 44，触限 0；自有 Java 句柄停止、OCI 状态消失、user-systemd unit inactive／dead。没有运行原 JAR、读正文或使用真实会话。[本机重建、失败、有限观测与清理](evidence/default-vue3-image-local-2026-10-06.json)。它不能替代真正完整镜像的双架构门禁。

## 尚未验证与已知故障

本修改自己的 GitHub hosted 双架构构建、重载、发布导入和完整镜像默认 UI 小报告尚未验收；已有的 Vue 3 显式选择旅程、旧镜像长测或普通工作流状态不能替代它们。静态 HTTP／字节检查不是登录点击、字体渲染、阅读交互或全部页面验收，仍须自己的 Vue 3 浏览器旅程。

此前 `08cccfca` 同镜像的起点匿名解析仍失败，本次不改浏览器业务源码，也不宣称修好认证／正文或原 JAR／历史远程服务的同输入三方兼容。静止内存增长与突发延迟的历史观察见[原制品长测](BROWSER-SOAK-2026-10-06.md)，不追认为新制品持续验证。生产仍未切换。

## 回退方法

完整候选镜像可设置 `READER_APP_WEBUI=vue2` 选择保留的原版资源，无需重建或存储迁移。Compose 同名环境变量／`.env` 也是明确选择；当前源码不自动修改服务器上的 `.env`。若要撤销产品默认变更，撤回 Docker ENV／Compose 选择及相关默认 UI 门禁，保留本轮失败与实际报告。切换生产仍须按正式发布流程进行备份、实际登录／阅读验收与镜像回滚，不把静态入口检查当作发布许可。
