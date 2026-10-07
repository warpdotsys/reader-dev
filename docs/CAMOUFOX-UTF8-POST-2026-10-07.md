# 内置引擎 UTF-8 POST 第 20 契约

## 改动及验收边界

只增加实际 Camoufox 契约、严格 XML 发布门禁，以及隔离三方工具的失败观察保留；没有改 worker、业务源码、前端、数据库或历史引擎。历史引擎真实 44／60 B 截断缺陷见 [三方诊断](WEBVIEW-UTF8-THREE-WAY-2026-10-07.md#单独历史引擎诊断及已确认旧缺陷)，不能为了追随缺陷而截断正确请求。

`generatedUtf8PostPreservesRawBytesAndSourceScriptResult` 通过固定版本真实浏览器发送生成 JSON：中文、`𠮷`、`😀` 和 `+ & %`。目标服务先保存收到的原字节，再解码；要求 POST 只发生一次、JSON Content-Type、Content-Length 60、原 60 B 完全一致、SHA-256 `8d0383070028f4f457194a194e8e9133df2487261e341c1da121171d59eeeef1`。请求字符串 UTF-16 长度为 44，不能用字符数代替网络字节数。源脚本必须返回完整 `WebView编码书𠮷😀 + & %`，原 HTML 的书名故意不同，不让未执行脚本的响应混过检查。

`verify-camoufox-contracts.py` 现在严格要求精确 **20 项、零失败／错误／跳过**，新增 `utf8PostContractPresent`。旧 19 项报告、伪造零计数但跳过第 20 项均被拒绝。旧报告只能用其冻结 Git 版本审计，不降低新门禁。

另新增 `three-way.historical-observation.json`：六用例的原件及恢复历史路径完成后，先独占创建观察文件，再执行原严格 handoff 验证。文件明确尚未评估接受／尚未执行 Camoufox；验证失败仍不能交接或生成兼容成功报告。存在的观察文件在启动任何 JAR 前拒绝覆盖。纯测试核对失败时只执行前两侧、保存实际字段且不运行 Camoufox。

## 已执行与尚未执行

- 本机 JDK 11 编译成功；新类 XML 精确 20 项，但本机无门控运行时，**20 项全部环境跳过，不是浏览器通过**。
- 本机纯 Python 全量 **259 项中 258 通过、1 Windows POSIX symlink 环境跳过**，0 失败／错误；新增 4 项覆盖门禁与观察保留。发布结构 36／原生身份 19／manifest 14 共 69 项纯测试分别实际通过；synthetic XML／mock 不等于真实浏览器。
- 新第 20 契约尚待本增量 GitHub 托管 runner 的实际 XML 验收，不能用前一提交的 19 项替代。
- 原件／历史／内置引擎的五例 logging-only 对照已实际通过；未加包装器的 c 轮在历史配对请求字段验证处失败，未交接 Camoufox。原返回没有提前落盘，不能补造；新增观察保留专门修复这个测试工具缺口，不修改原判定。[精确生成证据](evidence/webview-generated-controls-2026-10-07.json)。

本机可重复编译命令（项目自带 JDK 11、两个 worker、Gradle 堆 1 GiB）：

```powershell
$env:JAVA_HOME = (Resolve-Path '.tools/jdk-11.0.8').Path
.\gradlew.bat --no-daemon --max-workers=2 '-Dorg.gradle.jvmargs=-Xmx1024m' test --tests com.htmake.reader.utils.CamoufoxWebviewRendererTest
```

真正发布门禁由既有 `browser-image.yml`／`release-native.yml` 安装锁定运行时，实际执行测试，再调用 XML verifier；门控环境缺失必须拒绝，不能将上面的全跳过报告导出为发布接受。

## 前一诊断提交的独立托管验收

`450021a4`／受测 `be622eef8a75b25aa9d33946e87271854302637c` 的 [Java](https://github.com/warpdotsys/reader-dev/actions/runs/37594912598)、[Vue 3](https://github.com/warpdotsys/reader-dev/actions/runs/37594912652)、[完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/37594912539)、[原生演练](https://github.com/warpdotsys/reader-dev/actions/runs/37594912560)均终态 success；原生六个实际作业全部成功，Full 五个可选专项未执行。

只下载小报告重跑原守卫：两套真实 Camoufox 各 19、helper 各 3；普通 Java 162 中 30 门控跳过；Vue 核心 18＋子目录 1、几何 18／长组 3；publisher 精确 24 文件及 15 次原 UI／异步／预算守卫通过。最高 867,692,544 B／PID 204、触限／OOM／实际 swap 0，但 CI 允许 1 GiB swap，与服务器零 swap 不混。[独立证据](evidence/webview-utf8-hosted-450021a4-2026-10-07.json)。

共享原生 JAR `d128218d...`、Full `967e03e2...` 均不是服务器 221 输入，未下载新大镜像或新 JAR。58 份用户已有报告散列不变；未发正式版、写 registry 或改生产。

## 已知问题和回退

六用例完整三方仍未完成，旧历史 UTF-8 截断仍是已知缺陷；早先搜索超时原因未唯一定位。起点解析、真实认证三方、全部非 UTF-8 编码、当前候选长期容量及生产验证仍未完成。

回退只在干净检出撤销本测试／观察保留增量，不需要改业务数据或重建数据库；保持原 JAR 与此前失败证据。正常发布必须满足当前契约，不能为发版接受旧 19 项。不要 reset 用户工作区、覆盖观察文件或改旧引擎取绿灯。
