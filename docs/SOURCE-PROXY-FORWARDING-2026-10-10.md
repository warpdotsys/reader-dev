# 书源代理参数传递修复

日期：2026-10-10。范围为可读 Java/Kotlin 工程，不是语言重写。尚未部署或发版。

## 已从 JAR 验证

准确原件仍是 `D:\Download\reader-pro-3.2.14.jar`，72,913,887B，SHA-256 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`；只读副本同 SHA、只读属性保持。

仅提取两个类到新的忽略构建目录后用 JDK11 javap 核查。`AnalyzeUrl.getStrResponseAwait` 的 POST／GET 调用默认掩码分别为134／902；两者包含128。原 `ReaderAdapterInterface$DefaultImpls` 在128位将第8个参数proxy写为null。因此原件也有此参数缺口，不是根据相似源码认定版本相同。

## 已编译验证

当前源码在构造时把书源header中的proxy单独存储并从HTTP头删除；普通HTTP使用该值，但WebView两个分支未传入。新增测试实际运行URL规则解析器及ReaderAdapter，末端明确使用捕获请求的renderer替身，不伪装成浏览器实测，也不访问测试代理端口。

- 修复前：4项实际JVM测试、3项断言失败，接收的代理均为null；未配置代理的默认行为通过。原始失败XML另存，未覆盖为绿灯。
- 修复仅增加两个命名参数`proxy = proxy`。GET／POST、书源header、显式初始headerMap优先级、正文、脚本、sourceRegex、tag及用户命名空间均有断言；proxy不成为HTTP头。
- 修复后：JDK11／Gradle6.1.1离线重新编译，9项实际JVM测试／3套件、失败0／错误0／跳过0，包括新增4项、原适配器2项、Cookie兼容3项。

[独立核查回执](evidence/source-proxy-forwarding-2026-10-10.json)，SHA-256 `ad8afc94fdd53b287170c1a607c22b283d5ca5d6747e2e824309fbf1bebcd398`，包含两个原类、当前源码和实际修复前后XML的SHA。测试输出在独立构建目录，未执行clean，未清空现有测试输入；只删除每个JVM测试自己新建的临时目录。

这是有意的行为修正：原件对配置了代理的WebView静默直连，恢复版将遵从选定代理；无代理时仍为null。不能把这项差异修改进原件基准后称为完全一致。

## 尚未验证与已知问题

初次记录时只有参数替身。现在22f949cc自己的四条托管编译／Boot JAR／双原生结果已经独立接受；新fcd JAR本机生成Reader API经Java CONNECT及选定代理的8场景HTTPS实际完成，上游客户端逐socket核对属于该JVM。[新产物、实际观察与有限范围](READER-HTTPS-BUSINESS-2026-10-10.md)。新fcd另有4组／12份HTTP生成三方ReturnData相等，但不包括显式proxy差异、原件HTTPS、真实站点账号或生产接受。

[打包HTTPS门槛](PACKAGED-HTTPS-GATE-2026-10-10.md)的旧e723启动超时失败记录保留；22f949另加入最小私有app-data目录后自己的12例及两架构已经完成。proxy参数修复不是启动故障的解释，二者不可混淆。OPDS等既有未完成项继续保留，不因为有限子集通过而宣称整个项目完成。

## 可重复核验与回退

标准托管环境中可运行：

```sh
./gradlew test --tests com.htmake.reader.init.AnalyzeUrlWebviewProxyTest \
  --tests com.htmake.reader.init.ReaderAdapterWebviewTest \
  --tests com.htmake.reader.utils.WebviewCookieCompatibilityTest \
  --max-workers=2 --no-daemon --no-build-cache
```

本机为保护已有build内容，额外使用独立buildDir的初始化脚本及项目缓存，编译器限2逻辑CPU、768MiB堆，测试单进程256MiB堆。没有覆盖原始JAR或真实正文。

回退仅撤销`AnalyzeUrl`两处proxy命名参数以及对应新增测试，保留回执；不重加跨源认证头、不忽略TLS错误、不改存储或系统代理。撤销后上述3项会重新失败，应明确记为恢复旧的参数缺口，而非安全验收。
