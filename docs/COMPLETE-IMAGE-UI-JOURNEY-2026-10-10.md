# 完整镜像默认界面的生成数据验收

日期：2026-10-10。此增量新增验收，并修复镜像上传暂存目录映射；不改 JAR 业务、用户存储格式、UI 选择、内存/能力上限、原始 JAR 或用户数据。

## 证据边界

- **已核对源码与已有托管证据**：原有 25 项 Vue3 旅程的主要用例直接连接打包后的 Java/Kotlin JAR，不是 Vite 后端替代品。仅子目录部署用例使用预览服务器。已有成功不等于完整 Docker 镜像本身已经执行界面旅程。
- **本次新增，第三轮自己的托管实跑已接受**：`NativeImageDefaultUiTest` 直接连接本次构建的 `reader-browser-smoke` 容器 `http://127.0.0.1:18890`。不覆盖 `READER_APP_WEBUI`，并验证实际发布资源中的版本及完整 tested merge revision。原先默认 Vue3 HTML/静态资源与 JAR SHA 门禁仍在它之前执行。
- 本次生成 XML/PNG 头部单测仅验证校验器拒绝策略，绝不当作真实浏览器、截图像素或产品成功。

## 第三轮：上传修复后的完整镜像已通过

`a12abe6481534926e523f65872eabdd8677a7576` / tested merge `6429154d41d98407516c9f5a2702933b4a4f36ac` 的 [Full run 38025473013](https://github.com/warpdotsys/reader-dev/actions/runs/38025473013) 已成功，随后独立下载小回执，核对来源、唯一真实 XML、四张图、已有默认静态字节/异步/metadata/Cam20/Chrom23/helper3及最终累计预算。独立回执 SHA `94b6db644b53df37c00f895209c9d13e5ace2c916c8af627b5dc53b5d3126ae1`；校验执行源码 SHA `bc988b97acd47f72ad1906c744128c60fc068e43de492492a13914f2405ae9ed`。不借上一提交绿灯或覆盖前两次失败。

- 新旅程实际 1 次 / 8.316 秒 / 零失败、错误、跳过。真实上传的 `importBookPreview` 从上一轮500变为200 / `isSuccess=true`，随后旧 `saveBook`、中文两章阅读、翻章与刷新、退出及根页保护均经过。
- JAR 仍为 `e59a01bf190de757f73216557ff353411019507d25bad76f0c808b48366adb84` / 285,665,012 B；两次独立干净重建一致。Java run `38025472980` 的独立回执 `51bd72d7c5c36c6354e673c2f6886e6a7535c3fceed051f8dae5039876b3a460`，454 Python全部执行，181 JVM / 31原有环境跳过 / TTS编辑11全执行。Vue run `38025472963` 的25实际旅程零跳过也独立接受，回执 `79930af5b7f7d0a1817016a7716ddbd67d75f1b4f970cc801f2bcb50548aa66e`。
- 最终服务端峰值 845,193,216 B / PID192；CPU quota200000 / period100000，2GiB / 256PID。memory.max/OOM/oom_kill/PID触限均0，实际swap0；普通短测允许1GiB swap，memory.high仍为既有 `max`。客户端明确不属于此预算，不冒称全客户端加服务端2GiB或严格零swap长测。
- 主agent逐张核对图像：登录空表单、导入完成的弹窗及背景书架1本、第二章中文正文与控件、退出后的空登录界面均可辨，无真实账号或正文。登录图与上一轮同 SHA，已有像素图也核对；并非空白截图。导入截图仍保留可继续导入的完成弹窗；阅读入口使用旧返回bookUrl直接路由，**不额外声称所有书架卡片点击/弹窗行为也由此例验证**。
- 自己的 [Native run 38025472975](https://github.com/warpdotsys/reader-dev/actions/runs/38025472975) 六个实际作业及24个publisher JSON也已独立接受，回执 SHA `0b63ad5bc311ae6cf056b6ae3ed57f2e5ff13670a7e676bc5df57e07f6bf99d4`：双原生构建运行、各自重新导入并运行、同publisher导入器无注册表凭据的演练都经过。四份运行预算最高898,363,392 B／PID201、普通允许1GiB swap／实际0。新AMD64 image ID `sha256:bd36d843ff9e8664842569ebe69c6e3a9c861cf8c2d920a4bbe5026f17cd4cc7`，ARM64 `sha256:0e993e89c7faaa0a056493d090a87c7ca9db1212b4e6e1b37d577bb559bd8055`；仅下载小回执，不把托管归档散列冒充本机重算。本次新渲染旅程仅在Full的AMD64镜像上执行，不冒充ARM64渲染或本机新镜像结果。58份用户保护报告和原件仍保持原散列，生产、正式发布未变。

| 生成截图 | SHA-256 |
| --- | --- |
| login | `e137a1c0587fcd9001cb7fd233f7564bef7dd50a882d47c47b4c372b1be9da8a` |
| shelf | `f3c26c419e8010d634bfaf91530b13672451b470c4a9d51f37e2c6174a55f408` |
| reading | `589d1089815853d959042d142b6184d89ee444b42e4462f68a1353bbdbb7fd66` |
| logout | `e07bc5ac8fce8a97250149b4d73c886ce3d106152e0206097c28c43d548b6b09` |

## 实际旅程的必验项

1. 新建生成账号，验证注册后的命名空间令牌；退出后通过真实表单登录。
2. 错误密码必须是 HTTP 200 / `isSuccess=false` / 可见“密码错误”，不能产生可用会话。已经断言的提示通过真实关闭按钮移除，防止后续截图混入旧提示。
3. 正确密码、取消“记住”：令牌仅在 `sessionStorage`，刷新后仍能取得空书架。
4. 真实文件选择器导入仅两章生成中文 TXT，`importBookPreview` 后书架仍空；`saveBook` 后一条。保存路径必须属于旧后端 `storage/data/`，不调用 Rust 的 `uploadLocalBook`。
5. 读取中文首章，无替代字符；下一章的 `saveBookProgress` 成功、URL 章节变化、刷新仍显示第二章。返回首章与再次刷新也实际检查。
6. 返回书架后退出，两个存储区均无令牌；受保护首页重定向至登录界面。浏览器页面无异常，不得尝试访问非此 loopback origin。

登录空表单、导入后的书架、第二章阅读、退出后登录四张真实 1280×900 截图须保留。截图等待现有只读字体/有限动画就绪观察，不隐藏或加速产品动画。

## 可重复执行与失败策略

GitHub 托管 `Browser image integration` 的完整镜像启动后执行：

```bash
export READER_NATIVE_UI_ISOLATED=1
export READER_NATIVE_UI_URL=http://127.0.0.1:18890
export READER_NATIVE_UI_REVISION="$(git rev-parse --verify HEAD)"
export READER_NATIVE_UI_EVIDENCE_DIR="$RUNNER_TEMP/native-default-ui"
./gradlew -p browser-poc cleanTest test \
  --tests 'com.medwarp.reader.browserpoc.NativeImageDefaultUiTest' --no-daemon
python3 scripts/verify-native-default-ui-journey.py \
  --xml-directory browser-poc/build/test-results/test \
  --screenshots "$READER_NATIVE_UI_EVIDENCE_DIR" --revision "$READER_NATIVE_UI_REVISION"
```

执行依赖该流程创建的新存储、容器及托管 runner 的浏览器客户端。不得指向日常 Reader 或生产。一般单元构建可不启用该集成用例，但完整镜像流程单独校验唯一指定 XML，必须 1 次执行、0 跳过、0 失败、0 错误、有限非零耗时；缺 XML/截图或忽略错误都失败。先清理本模块的可重建测试结果，不能复用旧绿灯。

服务端继续使用原 2 CPU / 2 GiB / 256 PID 普通短测预算，允许 1 GiB swap 的既有政策不变；最终累计资源回执在 UI 旅程后收集，不重置计数器。**托管 Chromium 只是外部验收客户端，其内存不属于服务端 cgroup，因此不能宣称客户端加服务端合计仍限 2 GiB。** 产品内置 Camoufox 及依赖仍来自锁定镜像，不引入宿主 Chrome 的运行时依赖。

## 已知问题与未验证项

以下保留每轮发现及当时状态；上传修复的第三轮实证见上，不覆盖此前失败。错误日志脱敏、全部请求头差分、真实认证/长期/生产等尚未完成。

- 首次增量 `59468f03` / tested merge `438a7963` 的 Full run `38023674991` 实际失败：错误使用根工程 `:browser-poc:*` 任务，Gradle 报 `Project 'browser-poc' not found in root project 'reader'`。独立测试工程应使用 `-p browser-poc`，已修正并增加防回退检查；当时页面旅程尚未执行、无 UI XML/截图、没有最终累计预算，不能冒称登录失败或 UI 通过。该 run 保留红灯与原日志；Java `38023674968` 及 Vue `38023674838` 自己的回执已分别复验，但不代表 Full 通过。
- 第二轮 `6c8123f2` / tested merge `46d7eb8d` 的 Full run `38024428773` 实际执行一例/零跳过，5.463 秒后失败：生成注册、错误密码、正确登录、tab 会话刷新与空书架断言均经过，但 `importBookPreview` 返回 HTTP 500。JUnit 与服务器日志共同定位 `java.nio.file.AccessDeniedException: /app/file-uploads`。只保留一张登录截图，没有通过导入/阅读/退出或最终预算；不能把此前 25 项直连 JAR 的 UI 成功替代这个镜像缺陷。
- **新增已核对版本的修复，自己的实际镜像结果待验**：原件 `BOOT-INF/lib/vertx-web-3.8.5.jar` 为 285,581 B / SHA `da6a4a62cf429de538d04631a21d9e50796a1163321cafadeff7caabe6d724c3`，与只读核查缓存依赖严格同散列。`javap` 核对 `BodyHandlerImpl()` 硬编码 `file-uploads`，不存在可凭空假设的系统属性覆写；`BHandler.makeUploadDir` 先检查目标是否存在。镜像仅新增根所有者固定链接 `/app/file-uploads -> /tmp/reader-file-uploads`，原入口以 UID10001 创建私有0700目录，并拒绝暂存目录为外部链接、非自身所有或非私有模式。程序/JAR目录仍不可写，正式 storage 路径不变，未增加权限或字节码补丁。Full 与原生/转移 smoke 均实际检查链接、目录模式与身份。6 项本机 POSIX 暂存前置条件/自有目录双件检查通过，**不是 Vert.x HTTP 或 UID10001 的完整镜像成功**；68 项发布结构检查也通过，实际上传仍需自己的新 CI。
- 此轮生成 500 的旧错误日志包含未脱敏请求查询部分；未使用真实凭据，但说明生产错误日志的凭据脱敏仍需进一步检查修复。文档不保存生成令牌，也不把普通访问日志已有脱敏推断成所有错误日志安全。

- 准确原 JAR 三方测试中，固定历史 WebView 的中文 POST 请求字节仅 44/应有 60 B；严格 UTF-8 六例仍失败。不能以返回书名一致或本 UI 成功抹去它。
- OPDS 路由与独立账号仍未实现；前端明确提示不可用，不能伪造实现。
- 全部实际请求头/Content-Type 三方观测、真实书源登录态、长期资源与泄漏、全部本机渲染流程、生产部署仍未由此增量证明。
- 退出界面清除客户端会话不等于撤销所有既有 accessToken；保留原后端已实测的语义，不擅自改数据/认证。
- 未使用真实账号、已清除的起点 Cookie、私人正文、原浏览器会话。不购买章节，不发布或部署。

## 回退

失败时保留失败的 XML/生成截图与原始 JAR 对照；修复验收或实际缺陷后重新运行，不能删门禁或放宽预算冒充完成。上传修复是可读 Dockerfile/原入口源码，不是类字节码热补丁；回退至先前完整镜像可保留 storage 数据，但已知非root上传500也随之恢复，须明确提示。不要删除正在使用的暂存目录或修改程序根目录所有者。原 Vue2 回退选择和既有构建/发布回滚方法保持不变。
