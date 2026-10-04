# Netscape Cookie 导入修复与验收

## 实际发现与范围

用户提供了起点的 Netscape 导出。真实会话值不写入仓库、测试报告、服务器或 CI；本轮导入修复全部使用生成账号、生成 Cookie 和 `.example` / `.test` 测试域名。先前起点首页出现可见退出入口的证据仅证明当时网页认证，不能代替 Reader 验收，见[起点状态记录](QIDIAN-SOURCE-LOGIN-2026-10-04.md)。

旧本机构建实测拒绝多行导出，ReturnData 为 `isSuccess=false`、`errorMsg="Cookie 包含非法控制字符"`。该基线 JAR SHA-256 为 `BA1E3ACB91F04BD6A3A10C7406A76E11CB82C1BF3D41212D22D14BEF6953085C`；它是此前本机恢复构建，不是原始 3.2.14 JAR。旧后端的真实 Chrome 回归也失败，XML 保留在 `build/netscape-cookie-import-20261004-a/baseline-chrome-failure.xml`，SHA-256 `D35F5C5F64744618CF9595294DD2793D4F0235E66D1FCF20F200E29F4BD54BEC`。

另一个实际缺口是书源编辑使用单行 `<input>`。增强回归实测得到 `expected TEXTAREA, actual INPUT`，XML 保留为 `baseline-editor-single-line-failure.xml`，SHA-256 `D17ABAA28705D8EBC102C55861C78B61013DAE15FCE71C3DC7B233FC8B689777`；没有将第一次登录弹窗通过当作第二个入口也通过。

## 源码修复

- 按 [curl 的 Netscape 格式说明](https://curl.se/docs/http-cookies.html)读取七个 TAB 字段，保留末尾空值、CRLF、BOM、Domain/host-only、Path、Secure、到期时间及 `#HttpOnly_` 标记；0 表示会话 Cookie。格式没有 SameSite，不能声称恢复该属性或补造它。
- 前端登录弹窗和书源编辑均保留原始多行/TAB；编辑入口改为 textarea，不再先 `.trim()` 丢掉第七个空字段。
- 整份文件验证后才写入当前用户结构化 Jar；拒绝越域、公共后缀、无效前缀/布尔/时间/控制字符和超预算输入。当前上限为 65,536 个字符、512 条记录，错误只含行号和通用原因，不回显会话值。
- 保留 Secure、Path、HttpOnly 等限制，不复制成无属性扁平 Cookie。替换适用于所选主机的结构化凭据及旧手动覆盖；不复活该来源的旧扁平记录。旧 URL 缓存解析可能把 `example.co.uk` 合并到公共后缀 `co.uk`，此时只接管相应主机，不删或整体接管其他站点的共享后缀键。
- Cookie 列表显示“保存状态”，不以源站基地址的 Path/Secure 匹配判断是否已保存，也不把保存成功报成目标站已登录。摘要仅含名称与 `***`。

## 已实际重建与运行

最终本机 JAR：`build/netscape-cookie-import-20261004-a/candidate-multiline.jar`，SHA-256 `60C42281C7B12296C30DD4791750200E8CEAEAB520A1A3A547F48802DB7CDA41`。

使用新隔离存储、`secure=true`、`webUi=vue3`，只监听 `127.0.0.1:18940`。最终 Chrome 测试直接访问该 JAR 打包页面，不使用 Vite 预览；验证后停止本轮拥有的测试进程，保留生成证据和存储，用户 18931 端口未改动。

- 实际 API 返回 HTTP 200、`isSuccess=true`、空 errorMsg、`data={success:true,format:"netscape",imported:2}`。直接核对仅该新建测试用户的缓存，确认 Domain/host-only、Path、Secure、HttpOnly、会话到期值和空值实际落盘。
- 再提交“有效记录 + 越域错误记录”，实际 HTTP 200、`isSuccess=false`；错误不含凭据值，已有 Jar 文件 SHA-256 前后相同，脱敏摘要也不变。证明错误输入不造成部分导入；不将 HTTP 200 本身误当业务成功。
- 全量后端 36 个 suite / 123 项：0 失败/错误，21 项本机环境门控跳过。Cookie Jar 20 项及登录安全 3 项无跳过；包含公共后缀隔离、过期/重复、前缀和错误导入不改变已有状态。
- 前端 183 项全部通过；类型检查和生产构建通过。最终 JAR 的 `BOOT-INF/classes/web-vue3/static/SourceManageView-D9GH_R3f.js` 字节哈希与最新 dist 相同：`0D81A470D117FC286F382F71A578D6F43CE1C1FACBA4D0E28904F52281A0EBF8`。
- 同一条真实 Chrome 旅程最终为 1 项、0 跳过/失败/错误。覆盖两个导入入口的末尾空字段、实际 POST 返回 2 条、保存后隐藏输入、刷新/撤销/同域共享，以及通过界面实际退出登录后切换用户的隔离。不是单纯读取状态码或静态 DOM。
- 打包页面的 XML 与截图保留在 `build/netscape-cookie-import-20261004-a/packaged-chrome/`。XML SHA-256 `788697CAD19CE1A915C5F7CC9CA0DF90E3C37E519CEEC78592A814F28220059E`，截图 SHA-256 `22847CB01ED49EF060BE6A317FF0DAE4FFB5D0AC5BF57299FFEF22390A84BE83`；已目视核对中文和脱敏。

精简、无凭据的实际结果见[证据摘要](evidence/netscape-cookie-import-2026-10-04.json)。本机产物不是 GitHub 正式发版产物，不能与其他提交或服务器 JAR 的哈希互换。

## 本次源码的托管实跑：2cca1143

提交 `2cca1143a795537b3a26e293588e5cd59aaaa28b` 的 [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/37195595626)、[Vue 3](https://github.com/warpdotsys/reader-dev/actions/runs/37195595637)、[完整浏览器镜像](https://github.com/warpdotsys/reader-dev/actions/runs/37195595791)及[六作业原生传递演练](https://github.com/warpdotsys/reader-dev/actions/runs/37195595659)均已完成且成功。实际 PR 合并快照为 `a678bdbe75b2a15a6ab5a3acd1254f44c277c9d5`，不是已经合并到默认分支。

- 已下载的真实 Camoufox JUnit 为 12 项、0 跳过/失败/错误。新增 `importedNetscapeCookiesRetainScopeInRealBrowserRequests` 实际执行 7.727 秒：生成 Netscape Cookie 经过主请求与不同路径子资源；HttpOnly 对页面脚本不可见，HTTP 不发送 Secure，空值保留，其他用户不继承。不是仅编译或检查测试名称。
- 已下载的核心界面 JUnit 为 12 项、0 跳过/失败/错误，包含两个 Netscape 导入入口与实际界面退出/换账号隔离。此处为托管 Chromium，不是起点认证用例。
- 消费端 7,033 B 小型制品内的 20 份实际 JSON 已逐项核对：两架构共享同一 JAR，构建和新 runner 重载后的镜像、版本、请求、Cookie、四用户及资源字段符合预期。四次短时峰值 800,141,312–806,432,768 B，限额事件均为 0；swap 实际为 0，但允许上限为 1 GiB，不能描述成强制禁用 swap。完整镜像仍只在 GitHub runner 传递。

原始 JUnit 哈希、具体作业及边界见[本提交 Cookie 门禁摘要](evidence/netscape-cookie-ci-2cca1143-2026-10-04.json)和[双架构交付摘要](evidence/native-release-2cca1143-2026-10-04.json)。此前 `2caf3a5e` 的证据保持历史含义，不拿来证明此修改。

## 重复验证

安装仓库要求的 JDK 11、Node 24 后，在根目录：

```powershell
npm --prefix web-vue3 ci
npm --prefix web-vue3 test
npm --prefix web-vue3 run build
.\gradlew.bat -PreaderWebUi=vue3 test bootJar --no-daemon --max-workers=2
```

Chrome 旅程只对新隔离存储启动的回环 Reader 执行，显式选 `webUi=vue3`；不要指向用户正在使用的 18931 或生产：

```powershell
$env:READER_BROWSER_EXECUTABLE = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
$env:READER_VUE3_PREVIEW_URL = 'http://127.0.0.1:18940'
$env:READER_VUE3_ISOLATED = '1'
.\gradlew.bat -p browser-poc test --tests com.medwarp.reader.browserpoc.Vue3PreviewSourceCookieTest --rerun-tasks --no-daemon --max-workers=2
```

必须强制重新执行，否则 Gradle 的 UP-TO-DATE 不能证明新运行实例经过测试。

## 尚未验证与已知限制

- 本机未安装锁定 Camoufox 运行时，相关用例仍为本机门控跳过；上述 GitHub 托管实跑已补齐本提交的浏览器传输验收。两种运行环境不混淆。
- 真实起点 Reader 搜索、章节、认证三方尚未完成。已实际完成起点网站本身的认证、搜书和免费第一章显示，见[真实网站记录](QIDIAN-SOURCE-LOGIN-2026-10-04.md#同日真实网站搜书与免费章节验证)；不能拿它代替 Reader 或三方验收。
- 手动导入会替换相应已保存凭据；清空仍遵守 legacy 同域撤销语义。缓存和索引不是跨文件断电事务；文件系统写入失败及同时修改的并发语义未新增专项验收。
- 输入框粘贴阶段仍为明文。源站 Cookie 可能过期，验证码/扫码和收费权限不在本次修复范围；不绕过它们。
- 当前不合并、不升稳定版本、不发 Release、不推 registry、不改生产。旧 JAR、Vue 2 回退和用户未提交文件保持不变。
