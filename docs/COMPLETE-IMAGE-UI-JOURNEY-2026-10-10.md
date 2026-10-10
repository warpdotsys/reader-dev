# 完整镜像默认界面的生成数据验收

日期：2026-10-10。此增量不改业务、镜像默认值、原始 JAR 或用户数据。

## 证据边界

- **已核对源码与已有托管证据**：原有 25 项 Vue3 旅程的主要用例直接连接打包后的 Java/Kotlin JAR，不是 Vite 后端替代品。仅子目录部署用例使用预览服务器。已有成功不等于完整 Docker 镜像本身已经执行界面旅程。
- **本次新增，尚待自己的托管实跑**：`NativeImageDefaultUiTest` 直接连接本次构建的 `reader-browser-smoke` 容器 `http://127.0.0.1:18890`。不覆盖 `READER_APP_WEBUI`，并验证实际发布资源中的版本及完整 tested merge revision。原先默认 Vue3 HTML/静态资源与 JAR SHA 门禁仍在它之前执行。
- 本次生成 XML/PNG 头部单测仅验证校验器拒绝策略，绝不当作真实浏览器、截图像素或产品成功。

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

- 首次增量 `59468f03` / tested merge `438a7963` 的 Full run `38023674991` 实际失败：错误使用根工程 `:browser-poc:*` 任务，Gradle 报 `Project 'browser-poc' not found in root project 'reader'`。独立测试工程应使用 `-p browser-poc`，已修正并增加防回退检查；当时页面旅程尚未执行、无 UI XML/截图、没有最终累计预算，不能冒称登录失败或 UI 通过。该 run 保留红灯与原日志；Java `38023674968` 及 Vue `38023674838` 自己的回执已分别复验，但不代表 Full 通过。

- 准确原 JAR 三方测试中，固定历史 WebView 的中文 POST 请求字节仅 44/应有 60 B；严格 UTF-8 六例仍失败。不能以返回书名一致或本 UI 成功抹去它。
- OPDS 路由与独立账号仍未实现；前端明确提示不可用，不能伪造实现。
- 全部实际请求头/Content-Type 三方观测、真实书源登录态、长期资源与泄漏、全部本机渲染流程、生产部署仍未由此增量证明。
- 退出界面清除客户端会话不等于撤销所有既有 accessToken；保留原后端已实测的语义，不擅自改数据/认证。
- 未使用真实账号、已清除的起点 Cookie、私人正文、原浏览器会话。不购买章节，不发布或部署。

## 回退

此增量仅新增验收，不改产品层或业务。失败时保留失败的 XML/生成截图与原始 JAR 对照；修复验收或实际缺陷后重新运行，不能删门禁或放宽预算冒充完成。原 Vue2 回退选择和既有构建/发布回滚方法保持不变。
