# 错误日志保护与生成请求头观测

日期：2026-10-10。仅可读源码与隔离测试工具增量；不修改生产、原件、认证语义或存储格式。

## 已确认的问题与修改

此前完整镜像的真实上传500日志同时输出原异常和完整 `BasicError` JSON，后者包含查询中的生成 accessToken。普通访问日志的去查询处理并不能保护这一错误分支。

`RoutingContext.error` 现只写一条结构化记录：固定错误名、异常类型、HTTP方法、最多2048字符且去查询/片段/换行的路由、500状态和原时间戳。不向日志器传递原Throwable，不记录异常消息、原因链、suppressed异常或原堆栈文件名。不是仅用正则遮盖某一种token。

**兼容边界**：旧 HTTP 500 的状态、content-type、六个JSON字段及值保持不变；响应仍含原URI与异常消息。这次只修正已证实的日志出口，不能声称所有错误响应或其他日志均已脱敏。省略原堆栈和消息会减少诊断信息；应以类型、路由、时间与既有追踪标识关联，不能为了诊断把原异常重新加入日志。

## 本机已执行的验证

- JDK11、现有Gradle Wrapper，`RestVerticleLoggingTest` 实际6例／2.664秒／零失败、错误、跳过。新增三例在实际Logback收集器上检查查询令牌、消息、原因、suppressed、伪堆栈文件名、循环原因链、null消息、路由控制字符及长度；同时逐字段断言旧响应。另一个新增用例真正启动独立loopback Vert.x临时端口并返回500，实际HTTP响应保留旧值，收集的日志不含生成令牌或异常消息。
- 原始XML SHA-256：`7dd536c478ba43e7bf2a7bb3b1027d9b9cc407107262547eb1c5c6d5e67c1c73`。该定向测试不是原JAR黑盒差分，也不是完整产品长期运行。
- Python467例／26.865秒／通过，其中一个原有POSIX链接用例在Windows跳过。首轮465例有两项测试替身未声明新开关导致错误；已显式更新替身默认值，保留原超时阶段断言，不通过生产代码忽略缺失观测。不能把替身执行当真实三方结果。
- 发布结构检查68例通过。重新核对58份受保护报告，变化0；原JAR仍为 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`。

可重复执行：

```powershell
$env:JAVA_HOME = Join-Path (Get-Location) '.tools\jdk-11.0.8'
$env:GRADLE_USER_HOME = Join-Path (Get-Location) '.gradle-user-home'
.\gradlew.bat test --tests com.htmake.reader.verticle.RestVerticleLoggingTest --offline --no-daemon --max-workers=2
python -B -m unittest discover -s src/test/python -p '*_test.py'
node --test scripts/check-release-pipeline.test.mjs
```

## 请求头观测入口：可用工具，不是兼容结论

`compare-webview-cookie.py --observe-target-headers` 只允许现有的生成、隔离、实际三方GET/POST模式：准确原JAR、恢复JAR连接固定历史WebView、同恢复JAR使用Camoufox。原有原件身份与隔离门禁仍须通过；不允许用模拟 `/render.html` 替代目标站观测，不导入真实凭据。

目标fixture独立保存 `renderTargetHeaders`，包括所有解析后的字段对，保留大小写、顺序、重复字段与未知字段。每次最多64字段、名称和值合计最多8192 UTF-8字节；超限拒绝431，不截断后伪称完整。它不是HTTP原始线缆字节记录。旧 `renderRequestFields` 方法、正文、指定头及UTF-8原始字节散列的严格比较不变。

缺少参考侧的五次（编码模式六次）实际观测时，保存已取得的历史观察并拒绝启动Camoufox；最终三侧都必须完整。13个新增生成守卫验证此行为及CLI传递，不是实际浏览器请求证明。最终报告明确 `literalHeaderParityAccepted=false`；User-Agent、Accept、Content-Type等差异仍需真实执行后逐项审阅，不能把观测完整误称字面兼容。

本轮尚未用新JAR执行准确原件三方全请求头观测。此前已证的历史UTF-8 POST截断44／正确60字节、Cookie差异、真实认证和长测风险原样保留，不借此入口放宽。

## 自己的托管结果与回退

本增量提交后需由自己的GitHub托管runner重新构建、执行真实UI与完整镜像，再独立核对XML、同提交两次干净JAR字节、资源和双原生归档回执；不能借父版a12的绿灯或e59 JAR身份。本段在取得结果前明确待验。

测试工具回退可撤销新开关及独立观测字段，不改旧报告或现有断言。日志修改回退会重新引入查询令牌与异常消息泄露风险，不能无说明恢复。若回到父镜像a12，只使用其已核对身份并保留数据；它有上传修复，但不含本日志修复。新默认UI旅程、旧Vue2回退和所有既有资源门禁仍保留。
