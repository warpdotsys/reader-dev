# Vue 3 跨标签页鉴权隔离（2026-10-05）

## 证据边界

本轮仅使用全新 loopback Reader、两个生成账号、生成书架元数据与分组，不读取私人正文、不复用真实网站 Cookie、不改生产或用户现有 Reader。下面的基线是恢复工程候选，**不是原 Pro 3.2.14 JAR 的新差分**。原 JAR 保持只读。

## 已复现的缺陷

两个真实 Chrome 页面共享 Cookie，各自保留独立 `sessionStorage`，均不勾选“记住我”。乙登录后，甲页面仍显示甲，实际请求仍带甲的 token，却返回乙的书架并把它缓存到甲名下。HTTP 200 与 `ReturnData.isSuccess=true` 不能证明身份正确。

- 基线 JAR SHA-256：`7cd93beef1af6b0cfd1cc8759059cc48ffe01e8266507acfc3b2534ed581774f`。
- 基线 JUnit：1 项、0 跳过、1 失败、0 错误，34.761 秒。
- XML SHA-256：`ded31172f94e091ae04fbeb45a2be62837edada73454ff80a27e9308331e8670`。
- 本机冻结目录：`build/cross-tab-isolation-20261005-a/cross-before/`；保留 JAR、当时的测试源码、失败报告和生成截图。截图已目视确认用户名甲/书架乙。
- 源码解释：`BaseController.checkAuth` 先认可 Cookie，再检查 token；`getUserInfo`、`logout` 另直接读取 Cookie 身份。旧前端账号缓存名不能证明响应实际归属。

## 可读源码修复

1. Vue 3 的已认证请求显式携带 `readerAuth=access-token`，该模式只认可 token；无/错 token 不退回共享 Cookie。无此标记的旧客户端继续 Cookie 优先，非 secure 模式不改变。
2. 新模式普通鉴权不改绑 Cookie；历史 token 仍按原存储结构和到期规则续期。`getUserInfo` 使用验证后的 token 身份；退出甲 token 不销毁乙的 Cookie，也不撤销乙 token。
3. 公共请求、原生 SSE、EPUB 路径检查/下载、本人 PDF、封面与背景文件 URL 使用同一鉴权参数。资源 URL 重新显示时覆盖旧 token；新封面元数据不持久化 token。不向外站或图片代理追加凭据，检查规范化后的 URL 边界。
4. 章节、书架、阅读位置使用 `token-auth-v1` 版本化 scope；订阅镜像使用 `reader_source_subs_v2`，并绑定当前标签页内存身份与请求快照。旧全局/旧账号/旧 default 缓存保留，但不读、不迁、不删；需要在线重新建立可信缓存。服务端数据格式不转换。
5. `Vue3PreviewCrossTabTest` 已加入 GitHub 托管 runner 的强制门禁和生成截图归档；不能以条件跳过冒充通过。修复后测试加强了读写、身份接口、缺/错 token、旧 Cookie、历史 token、旧缓存拒读与退出隔离，基线冻结测试不改写。

## 当前验证状态

- 前端 217 项测试：0 跳过/失败/取消；类型检查和生产构建成功。
- 静态 API 映射：92/92 已注册；这不代表所有参数、权限和媒体格式已实测。
- 完整 Java/Kotlin JAR 重建成功；SHA-256 为 `4efb5598e505ddb8fbc316e4944b2a331ce39a524be6c7c83a7c8f36970d848e`。当前 81 个界面文件与包内逐字节一致；发布安全 47 项及工作流语法检查通过。
- 首轮候选 `cross-after`：1 项/0 跳过/1 失败/0 错误，53.096 秒。实际账号归属、读写、历史 token、缺/错 token、旧 Cookie 兼容及旧缓存拒读已通过；失败点为测试误把旧退出 `isSuccess` 预期成 false。报告 SHA-256 为 `0c92dea2b25033132b8e73c95a68909d7d662aa229c5e8abb97c490b7989f451`，目录冻结、不覆盖。
- **已从原 JAR 字节码验证**：`ReturnData.setData` 会设 `isSuccess=true`、默认 `errorMsg=""`；原 `UserController.logout` 在偏移 843/846 先设置错误，再在 849/855 设置 `NEED_LOGIN` 数据。已核对正常退出的三个 ReturnData 字段；候选的实际 HTTP 状态为 200，原 JAR 的 HTTP 状态本轮未重新黑盒探测。不能为了满足错误断言修改业务实现。用固定 JDK 的 `javap` 只读检查原 JAR 两个类；这不是新黑盒或三方差分。
- 已修正该断言并保持精确的四字段检查，使用同一候选 JAR 在新目录 `cross-after-b` 重测：**1 项、0 跳过/失败/错误，55.550 秒，全程终点完成**。XML SHA-256 为 `71635397078ed7fdb4f406b94f421d902a4a07ced0ff39396b478b9870df6c76`。实际甲 namespace、甲的页面/缓存、分组只写甲、缺/错 token 拒绝、旧 Cookie 仍认乙、历史 token 可用、旧错归属缓存拒读且原值保留、真实后端撤销甲历史 token 而乙继续使用，均通过。三个生成截图及首次候选的甲书架截图已目视核对；隔离进程正常清理。
- 同一候选 JAR 的核心 16 项真实浏览器全回归和管理空间/文件密钥 2 项已逐份 XML 核对通过：0 跳过/失败/错误。核心跨标签 41.443 秒、管理空间 22.533 秒、文件密码 7.426 秒；管理旅程完成 `credentialUrl=absent` 终点。目录分别为 `build/cross-tab-isolation-20261005-a/cross-core/`、`build/legacy-manager-20261005-a/cross-manager/`。

[本机结构化证据](evidence/vue3-cross-tab-local-2026-10-05.json)记录两份失败、最终候选 JAR/测试源码/报告身份与验证范围。

## 首轮托管结果及观察时序加强

源码 `386d9505fba8adbfc6c7c266bc43e12ced1793e6` 的 [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/37271310542) 和 [浏览器单镜像](https://github.com/warpdotsys/reader-dev/actions/runs/37271310538) 已成功；后者下载的 Camoufox 12 项 XML 无跳过/失败/错误，2 份实际单镜像 JSON 独立核对通过（2 CPU / 2 GiB / 256 PID，实测峰值 813273088 字节，实际 swap 0、配置允许 1 GiB swap）。[原生双架构演练](https://github.com/warpdotsys/reader-dev/actions/runs/37271310528) 六作业全部成功，20 份实际 JSON 的 JAR/镜像/归档身份、GET/POST、ReturnData、Cookie 顺序/隔离、并发和预算独立核对通过；源码对应 PR 合并修订 `64583f26037671cb2dd33f5d7f8790fa1d8e79f3`，共用托管 JAR `3251ea81c45e359dc42a48d1b255f2b769a02351afa95597ed9912293300bb74`。全部原生托管 runner，无自托管；最高实测内存 881381376 字节，无越限/OOM/PID 事件或实际 swap，但配置允许 swap。没有下载多 GiB 镜像到本机、发布 registry 或替换生产。上述成功不抵消下面的界面失败。

[Vue 3 首轮托管](https://github.com/warpdotsys/reader-dev/actions/runs/37271310498) **失败，不算整体验收**：核心 16 项中仅跨标签测试在断网恢复后立即检查书卡处失败（行 208，13.559 秒）。初始账号/响应/缓存隔离通过，但还未走完退出终点。下载的 XML 冻结于 `build/cross-tab-hosted-20261005-a/failed-core/`，报告 SHA-256 为 `301e885fea7378c7605b52bce84a2c1efee2177b6607d3e36fceacbce1c41438`。

旧测试以“当前没有 spinner”与两次绘制帧作为完成依据，不能证明随后请求及虚拟书卡渲染已完成。现保留所有业务断言，另要求恢复请求的实际状态码、ReturnData、namespace、书名及对应可见书卡，并在失败/成功时保存恢复截图；甲退出后的乙刷新也要求新的实际响应。**业务/前端代码和 JAR 未更改**，不以重试次数、固定等待或空实现掩盖失败。

加强后的 `cross-after-c` 本机 1 项/0 跳过/失败/错误，53.981 秒；真实恢复响应与实际书卡、甲退出后的乙新请求均达到终点，两个新截图已目视。同源码/同 JAR 在新目录 `cross-after-d` 独立再过：50.755 秒，0 跳过/失败/错误，终点完整（XML SHA-256 `c2b57a271ee98a6035bd459da26209d1c3d022736dfd3dc407c8206f3e7f7373`）。旧失败始终保留，不改标成功。

## 加强后的源码已托管验收

源码 `15d0186e2dc0b9954ebf896667eecf9b64ab43ce`、实际 PR 合并修订 `9ef38a3ca32629b138e7c5205d8bb1cb4a4475e0` 的四条工作流全部成功：[Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/37272784257)、[Vue 3](https://github.com/warpdotsys/reader-dev/actions/runs/37272784265)、[浏览器单镜像](https://github.com/warpdotsys/reader-dev/actions/runs/37272784270)、[原生双架构六作业演练](https://github.com/warpdotsys/reader-dev/actions/runs/37272784271)。全部 GitHub 原生托管 runner，不使用自托管。

下载后逐份检查核心 16、管理/文件 2、子目录 1 个实际 XML，共 19 项，0 跳过/失败/错误。跨标签 15.572 秒，XML SHA-256 为 `81c1f7ca48cb2122ccc1c92dff6b2ae42434f7f911f9d8de26d4142b430fc785`，两个实际刷新响应/书卡和全程终点均达到。五份生成截图已目视，甲/乙各自书架、旧缓存拒读通过；恢复截图仍可见先前 Network Error 提示，不宣称提示问题全修。

Camoufox 12 项实际 XML 无跳过/失败/错误；2 份单镜像和 20 份原生/重新导入 JSON 独立核对 GET/POST、ReturnData、Cookie 顺序/隔离、JAR/归档/镜像身份与资源预算。双架构共用托管 JAR SHA-256 为 `a720f596e7f67020b042099ba39005459a827076699a3f65e667369d72521e7b`，最高实测内存 833818624 字节（约 795 MiB），2 CPU / 2 GiB / 256 PID，无触限/OOM/PID 事件、实际 swap 0；配置仍允许 1 GiB swap。没有下载多 GiB 镜像到本机、发布 registry、合并 PR、创建 Release 或替换生产。

[结构化托管证据](evidence/vue3-cross-tab-hosted-15d0186e-2026-10-05.json)绑定源码、合并修订、实际报告散列和制品 ID。这是该提交的短时生成测试，不验证随后源码变更、真实认证三方或长期/生产范围。该提交的退出仍是单独调用后端后再清本地；按钮本身服务端撤销及混合“记住我”的后续工作必须另验。

## 已知限制与回滚

- 本轮真实多标签测试覆盖“不记住我”的独立账号。混合记住/不记住、标签页刷新后恢复身份及完整会话同步仍需真实界面测试；订阅镜像的内存身份边界有单元覆盖，不能冒充这些完整旅程。
- Vue 3 的退出按钮目前仅清除本机身份；本轮后端撤销测试单独调用实际 `POST /reader3/logout` 后再操作退出按钮，不将本机清除当成已撤销服务端 token。
- 系统 PDF 直开仍关闭；系统订阅离线仍关闭；系统空间完整 EPUB/漫画/音视频/SSE、真实网站认证三方、生产反代头、长期负载与最终镜像本机运行仍待验收。
- 断网重试提示仍可能叠加。没有将所有界面问题宣称修好。
- 不合并 PR、不创建正式 Release、不发布 registry、不替换生产，不清理用户未提交文件。
- 新界面与修复后端应作为同一个 JAR/镜像交付，不把新静态资源单独接到不识别新模式的旧后端。必要时回退整个已知发布产物及 Vue 2 入口；不要恢复旧缓存的可信归属或 reset 工作区。

重复构建：先在 `web-vue3` 运行 `npm ci`、`npm test`、`npm run build`；再在工程根运行 `gradlew.bat -PreaderWebUi=vue3 bootJar --no-daemon`。本机使用固定 JDK 11.0.8、离线依赖、2 工作线程、构建堆 1536 MiB；隔离 Reader 堆 512 MiB、2 个 JVM 可见处理器，不等于 Windows CPU/RSS 硬配额。
