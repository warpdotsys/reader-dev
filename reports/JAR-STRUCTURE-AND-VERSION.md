# reader-pro-3.2.14.jar 只读取证报告

取证日期：2026-09-19（Asia/Shanghai）。本报告的所有结论来自对原始 JAR 的只读 ZIP/字节码检查；未修改、移动或覆盖 `D:\Download\reader-pro-3.2.14.jar`，也未修改任何既有工程。

## 1. 原件标识与备份前校验

| 项目 | 已从 JAR 验证的值 |
| --- | --- |
| 原件 | `D:\Download\reader-pro-3.2.14.jar` |
| 长度 | 72,913,887 bytes |
| 文件时间（UTC） | 2026-07-21T01:56:41.5731400Z |
| SHA-256 | `B26FB4769D689D98FF26408CE79A275D719F360906C84ACF52FF404E98030C8C` |

校验命令（仅读取）：

```powershell
Get-FileHash -Algorithm SHA256 -LiteralPath 'D:\Download\reader-pro-3.2.14.jar'
```

本子任务按边界没有复制 JAR；项目主任务应以该散列校验其只读备份。

## 2. 可执行包与启动方式

JAR 内共 1,340 个 ZIP 条目，是 Spring Boot 2.1.6.RELEASE 的可执行 fat JAR。`META-INF/MANIFEST.MF` 已验证如下：

```text
Main-Class: org.springframework.boot.loader.JarLauncher
Start-Class: com.htmake.reader.ReaderApplicationKt
Spring-Boot-Version: 2.1.6.RELEASE
Spring-Boot-Classes: BOOT-INF/classes/
Spring-Boot-Lib: BOOT-INF/lib/
```

因此标准启动命令是：

```powershell
java -jar 'D:\Download\reader-pro-3.2.14.jar'
```

当前机器的 `PATH` 中未发现 `java`、`jar`、`javap` 或 `jdeps`，常见 JDK 安装目录也未发现 `javap.exe`；因此本子任务只完成了静态取证，**尚未在此机器上启动原 JAR**。后续首轮黑盒差分前需要先提供/安装兼容 JDK（建议先以 JDK 8 运行，因应用 class major 为 52）。

真实应用入口是 Kotlin 文件门面类 `com.htmake.reader.ReaderApplicationKt`，不是 Spring MVC/WAR 部署。应用通过 Spring Boot 启动，包内同时含 Vert.x 3.8.5 和协程依赖；从业务类名及依赖可确定路由层主要是 Vert.x，而不是仅靠 Spring MVC。

ZIP 区段（条目数 / 未压缩大小）：

| 区段 | 条目数 | 未压缩大小 |
| --- | ---: | ---: |
| `BOOT-INF/classes` | 1,193 | 55,959,891 |
| `BOOT-INF/lib` | 86 | 38,649,132 |
| Spring Boot loader 类 | 55 | 176,045 |
| `META-INF` | 2 | 242 |

## 3. 字节码、语言与可反编译性

| 检查项 | 已从 JAR 验证的结果 | 含义 |
| --- | --- | --- |
| 应用 class 数 | 936 | 不含嵌套依赖 JAR |
| class major | 52：936/936 | 全部为 Java 8 目标字节码；恢复构建至少需 JDK 8 兼容目标 |
| Kotlin `Metadata` 注解 | 754/936 | 主体为 Kotlin，但有 Java 代码/第三方源码一并打入 classes |
| `SourceFile` | 936/936 | 全部保留源文件名线索 |
| `LineNumberTable` | 755/936 | 大多数 class 可用于反编译结果和堆栈行的人工校准；部分协程状态机/合成类没有行表 |
| Kotlin module | `BOOT-INF/classes/META-INF/reader-pro.kotlin_module`（1,005 bytes） | 仍保留 Kotlin 模块索引 |

包内 Kotlin runtime 是 `kotlin-stdlib/reflect` 1.5.21、`kotlin-stdlib-common` 1.5.30，协程 1.5.2；这能可靠约束恢复工程的 Kotlin 线，但**尚不能单凭 runtime JAR 证明原始 Kotlin Gradle 插件精确版本**。应先从最接近源码的 Gradle 文件验证插件与 Wrapper，再以本 JAR 的 ABI/行为做差分。

特别关键的、具备行号和 Kotlin 元数据的类包括：

```text
com.htmake.reader.ReaderApplicationKt
com.htmake.reader.ReaderApplication
com.htmake.reader.api.YueduApi
com.htmake.reader.api.ReturnData
com.htmake.reader.api.controller.{BaseController,BookController,BookSourceController,
  BookGroupController,BookmarkController,FileController,HttpTTSController,
  LicenseController,ReplaceRuleController,RssSourceController,UserController,WebdavController}
```

`YueduApi` 含大量 `initRouter$N` 合成 lambda（可见编号至少至 138），说明接口恢复应优先用近似源码对齐，再用 CFR/Vineflower 的单类结果人工还原；不应把协程 state-machine 伪代码直接提交为业务源码。

## 4. 已验证依赖版本

以下是 `BOOT-INF/lib` 的完整文件名清单（86 个依赖 JAR，含一个目录条目未列）。文件名和其中可用的 `pom.properties` 均已读取；未从名称推测版本。

```text
accessors-smart 2.4.7; animal-sniffer-annotations 1.17; annotations 13.0;
antlr4-runtime 4.7.2; asm 9.1; bson 3.8.2; checker-qual 2.8.1;
commons-lang3 3.8.1; commons-logging 1.2; error_prone_annotations 2.3.2;
failureaccess 1.0.1; fontbox 2.0.27; gson 2.8.5; guava 28.0-jre;
hutool-core/hutool-crypto 5.8.0.M1; jackson-annotations 2.9.0;
jackson-core/jackson-databind 2.9.9; jackson-module-kotlin 2.13.5;
javax.annotation-api 1.3.2; json-path 2.6.0; json-smart 2.4.7;
jsoup 1.14.1; JsoupXpath 2.5.0; jsr305 3.0.2; kotlin-logging 1.6.24;
kotlin-reflect/kotlin-stdlib/kotlin-stdlib-jdk7/kotlin-stdlib-jdk8 1.5.21;
kotlin-stdlib-common 1.5.30; kotlinx-coroutines-core-jvm/slf4j 1.5.2;
log4j-api/log4j-to-slf4j 2.11.2; logback-classic/core 1.2.3;
slf4j-api/jul-to-slf4j 1.7.26; mongodb-driver-core/sync 3.8.2;
netty-{buffer,codec,codec-dns,codec-http,codec-http2,codec-socks,common,handler,
handler-proxy,resolver,resolver-dns,transport} 4.1.42.Final; okhttp 4.9.1;
okio-jvm 2.8.0; pdfbox 2.0.27; retrofit 2.6.1; retrofit-vertx 1.1.3;
rhino 1.7.13-1; snakeyaml 1.23; Spring Framework {aop,beans,context,core,
expression,jcl} 5.1.8.RELEASE; spring-boot/autoconfigure 2.1.6.RELEASE;
spring-boot-starter 2.1.6.RELEASE; spring-boot-starter-logging 2.1.6.RELEASE;
sysout-over-slf4j 1.0.2; vertx-{auth-common,bridge-common,core,lang-kotlin,
lang-kotlin-coroutines,web,web-client,web-common} 3.8.5; xmlpull 1.1.3.1.
```

其它可验证的依赖：`logging-interceptor 4.1.0`、`listenablefuture 9999.0-empty-to-avoid-conflict-with-guava`、`j2objc-annotations 1.3`、`xmlpull 1.1.3.1`、`okio-jvm 2.8.0`。恢复时须显式锁定这些版本，尤其不要让 Spring Boot 2.1 的依赖管理把 JAR 中已混用的 Jackson 2.9.x / jackson-module-kotlin 2.13.5 静默改写。

未在 JAR 中找到 Gradle Wrapper、`gradle-wrapper.properties`、`build.gradle(.kts)` 或 Maven POM；故 Wrapper/Gradle 版本目前为“尚未从 JAR 验证”，必须由接近源码或可复现构建试验确定。

## 5. 配置、存储与静态资源

`application.yml` 是包内默认配置，关键的已验证默认值：`reader.app.workDir: "."`、`reader.server.port: 8080`、`contextPath: ""`、`packaged: false`、`userLimit: 15`、`userBookLimit: 200`、`mongoDbName: reader`、`defaultUserEnable{Webdav,LocalStore,BookSource,RssSource}: true`。日志默认路径为 `${reader.app.workDir}/logs`，生产 profile 使用按日滚动且保留 7 天。

资源总览：

| 路径 | 文件数 | 未压缩大小 | 结论 |
| --- | ---: | ---: | --- |
| `BOOT-INF/classes/web` | 70 | 7,594,356 | 主 Web 前端；含 `index.html`、JS/CSS、字体、图标、PWA/service worker |
| `BOOT-INF/classes/simple-web` | 30 | 41,623,475 | 另一套静态 Web 资源（`assets` 25 项） |
| `dtd` | 66 | 334,013 | 解析相关资源 |
| `epub` | 7 | 117,639 | EPUB 资源 |
| `defaultData` | 2 | 5,967 | 默认数据 |
| `images` | 2 | 1,988,111 | 图像资源 |
| `bookSourceDebug` | 4 | 47,891 | 书源调试页面资源 |

主前端根目录已验证包含：`index.html`（5,625 bytes）、`favicon.ico`、`browsertest.html`、`service-worker.js`、`manifest.json`、`precache-manifest.0d903434eaa73f94acefeef5d39c6628.js`、`sw.js` 和 `robots.txt`。第一阶段恢复后端时可直接保留/复制这些原始资源来确保 UI 不先发生漂移。

## 6. 恢复风险与可恢复程度（仅限本 JAR 证据）

- **高可恢复**：启动骨架、依赖坐标大部分、配置默认值、静态前端、路由/控制器类名、Kotlin 源文件名与大部分调试行号。
- **中等风险**：Kotlin 协程 lambda、内联函数、扩展函数及泛型。元数据有助于还原签名，但 CFR/Vineflower 输出必须人工整理并与近似源码/黑盒响应核对。
- **高风险/尚未验证**：精确 Gradle Wrapper 与 Kotlin 插件版本、构建脚本中的自定义任务、运行期外部工作目录的 storage/data 布局、全部路由行为和认证/SSE 边界。它们不在 manifest/依赖名称中，不能由本次静态取证声称已恢复。

## 7. 可复核命令

```powershell
# 读取清单（不解包、不改写 JAR）
Add-Type -AssemblyName System.IO.Compression.FileSystem
$z=[System.IO.Compression.ZipFile]::OpenRead('D:\Download\reader-pro-3.2.14.jar')
$e=$z.GetEntry('META-INF/MANIFEST.MF')
$r=[IO.StreamReader]::new($e.Open()); $r.ReadToEnd(); $r.Dispose(); $z.Dispose()

# 以 Java 8 目标检查一个 class 的 major（需要将该 class 只读提取到隔离临时目录后用 javap）
javap -verbose -classpath reader-pro-3.2.14.jar com.htmake.reader.ReaderApplicationKt
```
