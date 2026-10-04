# 书源弹窗键盘操作修复与实际回归

## 从恢复 JAR 实际复现（不是原始 3.2.14）

本机隔离 Chrome 先对已构建的 `candidate-multiline.jar` 执行新增回归。JAR SHA-256 为 `60C42281C7B12296C30DD4791750200E8CEAEAB520A1A3A547F48802DB7CDA41`，不是原始 3.2.14 的散列。实际失败为 `Opening 书源登录 must move focus into the dialog`：点击登录按钮后焦点未进入弹窗。基线 XML 为 1 项、1 失败、0 跳过/错误，保留在 `build/source-dialog-keyboard-20261004-a/baseline-keyboard-failure.xml`。

第一轮修复 JAR 已通过登录、编辑、Cookie 管理及其他窗口的检查，但新增的本地文件导入预览失败：`Closing 导入本地书源 must restore focus to its opener`。没有删掉这个断言或把局部成功算成整条通过。失败 JAR（`7846BF23...`）与 XML `first-fix-preview-failure.xml` 继续保留；完整散列见[证据摘要](evidence/source-dialog-keyboard-2026-10-04.json)。

## 源码确认与修复

- 原自定义窗口只有 ARIA 标签和局部 Esc 处理，缺少打开时聚焦、Tab 首尾循环及关闭后恢复。新增 `dialogFocus` 指令，接入书源页九个自定义 dialog/alertdialog；动态查询可用控件，排除隐藏、禁用和不可 Tab 控件，清理文档监听，不覆盖原有业务关闭/忙碌规则。
- 自定义窗口嵌套时只让顶层管理焦点。Element Plus 的外部模态框保留自身焦点管理；相关单元测试通过，不冒充真实外部嵌套框已经全面验收。
- 文件选择器和异步读取会让当前 activeElement 不再是原按钮。本地导入入口显式保存触发按钮，并将其传给预览窗口；关闭恢复目标不再错误地依赖隐藏 file input。新增单元测试和真实 FileChooser 回归均通过。
- 编辑窗原 `closeEdit`、卸载和账号/默认命名空间切换没有清理 `editCookie`。修复后清理临时输入，切换身份还关闭编辑窗；不把 Cookie 放进书源 JSON、不改变后端请求契约。关闭/重开为空经实际 Chrome 验证，切换期间在途请求和浏览器内存取证仍未验证。

## 已成功重建与实际界面验证

最终本机 JAR 为 `build/source-dialog-keyboard-20261004-a/candidate-dialog-focus-v2.jar`，SHA-256 `CE0A868B0097D051828B7A3E59DD55BB0E32643C98C3E8ADF2EEF46731634326`。独立存储只监听 `127.0.0.1:18942`，启用安全登录、禁用许可证检查、选择 Vue 3，JVM 限 768 MiB 和 2 个可用处理器。测试使用生成账号、生成凭据、`.example` 域名和内存生成 JSON；未读取真实书籍或使用起点 Cookie。

- 前端全量 192 项、0 失败/跳过；焦点专项 9 项均通过。类型检查、生产构建通过。资源校验 9 项通过，OOM 负向样本明确带 `max=0`，避免仅因缺少计数而产生假阳性。
- JDK 11/Gradle Wrapper 全量重编译实际执行五项任务，4 分 40 秒成功；第二次仅前端变化，类型检查/构建后资源和 bootJar 实际重新执行，Java/Kotlin 两项保持 up-to-date。没有将 up-to-date 冒充第二次重编译。
- 最终 JAR 的 `BOOT-INF/classes/web-vue3/static/SourceManageView-CtPlIRxN.js` 与最新 dist 的字节散列相同，为 `D6D98A1234A78149C2E93037EFA10F7CA2634F8BD837E7C553CD65935CCCB496`。
- Chrome 直接访问打包 JAR，不是 Vite 页面。完整旅程为 1 项、0 失败/错误/跳过，实际 25.725 秒；覆盖登录/编辑两入口多行 Netscape（含末尾空字段）、返回 2 条导入、保存后隐藏凭据、刷新/撤销、真实界面退出后换账号隔离，以及以下窗口的焦点进入、Tab/Shift+Tab 循环、Esc 关闭和返回按钮：登录、编辑、Cookie 管理、新增书源、远程导入、书源调试、删除书源（仅取消）和本地导入预览（仅取消，确认未导入生成书源）。
- 登录/编辑在桌面运行；Cookie 管理及其后的新增/远程导入/调试/删除/本地预览在 375×812 窄屏运行。登录与窄屏 Cookie 截图已目视检查中文和布局；不以单张截图证明整条操作。
- 发布结构/原生制品相关 33 项本机回归通过，工作流 actionlint 通过（未启用外部 shellcheck/pyflakes）。托管工作流现在保留三张额外的**生成数据**键盘截图；新提交的托管结果还须独立验收，不能沿用 `868654e4` 的绿灯。

测试后仅停止本轮确认身份的 18942 测试 JVM，保留失败/成功 JAR、XML、截图与生成存储，18942 不再监听；用户的 18931/PID 61244 仍监听。没有改生产、用户报告或原 JAR。

### 可重复验证入口

在具备锁定工具链的干净检出中：

```powershell
npm ci --prefix web-vue3
npm test --prefix web-vue3
npm run build --prefix web-vue3
.\gradlew.bat bootJar -PreaderWebUi=vue3 --no-daemon --max-workers=2
```

在新建、回环监听的隔离 Reader 启动后，设置 `READER_VUE3_PREVIEW_URL`、`READER_VUE3_ISOLATED=1`、`READER_BROWSER_EXECUTABLE`（真实 Chrome/Chromium 路径），然后执行：

```powershell
.\gradlew.bat -p browser-poc test --tests 'com.medwarp.reader.browserpoc.Vue3PreviewSourceCookieTest' --no-daemon --max-workers=2 --rerun-tasks
```

必须检查 XML 为 `tests=1, skipped=0, failures=0, errors=0`。未设置隔离环境时的跳过不算成功；禁止指向生产或 18931。

## 已知限制、未验证与回滚

九个书源自定义弹窗已接入，但本轮真实键盘旅程覆盖八类；“删除订阅”及外部嵌套组件仍未有专项真实键盘测试。其他页面的自定义弹窗、读屏器、完整移动触摸/系统软键盘也不能由此推断通过。关闭重开空输入不是内存擦除证明；真实起点认证解析、付费权限、完整三方对照、正式发版和生产部署仍未完成。

回滚在干净维护分支上撤回本增量、重新构建已验收前一快照，或在隔离场景切回保留的前一 JAR；不要在用户脏工作区 reset/checkout 覆盖。没有存储迁移或字节码补丁；源码修复只涉及焦点、临时输入和测试/文档。正式产物仍需托管 runner 构建和验收，不能直接用本机 JAR 替换生产。
