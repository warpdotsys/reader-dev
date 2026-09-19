# Java/Kotlin 反编译可恢复性评估（仅对原始 JAR 读取）

## 已从 JAR 验证

- 样本：`D:\Download\reader-pro-3.2.14.jar`；SHA-256 为 `B26FB4769D689D98FF26408CE79A275D719F360906C84ACF52FF404E98030C8C`。
- 这是 Spring Boot 可执行 JAR：`Main-Class=org.springframework.boot.loader.JarLauncher`，`Start-Class=com.htmake.reader.ReaderApplicationKt`，`Spring-Boot-Version=2.1.6.RELEASE`。
- 反编译抽样脚本的归档口径扫描到 986 个 class（包含 Spring Boot loader）；独立 JAR 结构核查确认其中 `BOOT-INF/classes` 下有 936 个应用 class，936/936 均为 class-file major 52（Java 8）。包内有 `reader-pro.kotlin_module` 和 Kotlin 1.5.21/协程 1.5.2/Vert.x Kotlin 3.8.5 依赖；即“运行时 Java 8 字节码 + Kotlin 1.5 编译器产物”。
- 八个关键样本（启动、`ReturnData`、路由、用户控制器、用户模型、网络书源）都有 `kotlin.Metadata`、`SourceFile`、`LineNumberTable` 和 `LocalVariableTable`。`ReturnData` 的 Metadata `mv=[1,5,*]`，源文件名为 `ReturnData.kt`，且 `reader-pro.kotlin_module` 在包内。因此可用 Metadata、行号、局部变量名反向校验人工整理，而不只是猜测。
- 路由/控制器/书源样本含 `Continuation`（suspend 状态机）；`ReturnData`、`BaseController`、`UserController`、`User`、`WebBook` 含 `$default` 桥接方法。`User` 是 Kotlin data class（component/copy/default-constructor 产物）。这些都不能机械改写成 Java 或把反编译输出直接编进项目。

完整清单见 `jar-inventory.txt`；样本字节码、`javap -v` 输出分别在 `samples/`、`javap/`。

## 工具与对照

本机 PATH 没有 Java；为避免全局安装，评估使用已存在的只读 JDK：

| 工具 | 实际版本 | 用途/结论 |
|---|---:|---|
| Temurin JDK | 17.0.19（另发现 JDK 8u492、11.0.31） | `javap -p -v` 能稳定给出 ABI、泛型签名、注解、行号、局部变量与原始字节码；它是唯一可作为语义校验底稿的工具。 |
| CFR | 0.152 | Java 伪源码可读性对简单 DTO/启动类较好；对 Kotlin coroutine/default-arg/controller 有类型冲突、缺类和不可实现的 default super-call。 |
| Vineflower | 1.10.1 | 可以还原更多控制流，但其 Kotlin 模式输出出现不合法的伪 Kotlin import（如 `YueduApi.getSystemInfo.1`），并至少有一个方法直接 `Couldn't be decompiled`。 |

反编译命令可复跑：

```powershell
Set-Location C:\Users\chong\Documents\Codex\2026-09-16\g-i-t
.\work\recovery-analysis\decompile-eval\inspect-jar.ps1
.\work\recovery-analysis\decompile-eval\extract-samples.ps1
.\work\recovery-analysis\decompile-eval\run-decompilers.ps1
```

其中使用的 CFR/Vineflower JAR 固定在本目录 `tools/`；输出只用于人工恢复证据，**不得纳入正式 `src/` 或当作可构建源码**。

## 抽样证据与质量结论

| 样本 | 结构性证据 | CFR | Vineflower | 恢复建议 |
|---|---|---|---|---|
| `ReaderApplicationKt` | `ReaderApplication.kt`、无 suspend | 38 行，可作启动配置核对 | 16 行，较简洁 | 可用近似真实源码为主、`javap` 校验入口。 |
| `ReturnData` | 3 字段、`setData$default`、完整行号/参数名 | 63 行，字段/默认值可读 | 24 行，基本可读 | 高可恢复；以 ABI + JSON 黑盒确认 `isSuccess/errorMsg/data` 默认值。 |
| `User` | Kotlin data class、13 参数 copy/default 构造 | 417 行 | 249 行 | 中高可恢复；不得误删 Jackson 可见字段命名或默认构造。 |
| `YueduApi` | 3 个公开 suspend 方法、`initRouter$suspendImpl`、大量路由 lambda | 1013 行，开头列出许多未解析依赖 | 386 行，但含无效伪 Kotlin import 和状态机变量 | 仅用作路由/字符串/调用图证据；应以近似源码逐路由比对。 |
| `BaseController` | session、namespace、storage、WebDAV、并发限制均为 suspend/默认参数混合 | 有 `Could not resolve type clashes` 和 `Super calls with default arguments not supported` | 至少一个方法 `Couldn't be decompiled` | 高风险，必须基于真实源码人工恢复，逐方法 `javap`/黑盒核对。 |
| `UserController` | login/accessToken/文件/备份均 suspend | 1754 行，反复类型冲突 | 1535 行；可读字符串和分支较多，但仍是状态机伪码 | 高风险；反编译只可定位差异，不能直接采用。 |
| `WebBook` | 多个 suspend + `$default`，包含书源搜索/目录/正文 | 885 行 | 859 行 | 中等风险；可提取业务调用/常量，需同源代码和集成测试恢复。 |

Vineflower 日志明确报出 `ERROR: Statement cannot be decomposed although reducible!`；输出 `BaseController.java` 第 472 行保留 `// $VF: Couldn't be decompiled`。CFR 的 `BaseController` 又明确保留 `Super calls with default arguments not supported in this target`。这两项是实证，不是工具偏好判断。

## 可恢复程度与建议流程

整体判断：**若能找到接近 JAR 的真实 Kotlin/Java 源码，恢复可行性高；仅依赖反编译则核心后端只能达到中低置信度，不适合直接进入长期维护。** 静态资源、依赖坐标、ABI、路由文本、默认值和多数 DTO 可直接从 JAR 取证；协程状态机、默认参数、嵌套 lambda、Kotlin nullability、Spring/Vert.x 注入边界则会在反编译中显著损伤。

推荐顺序：

1. 以版本最接近的真实源码作为唯一可维护骨架，保留其 Kotlin 文件与协程写法。
2. 用 JAR 的 `javap -p -v` 对每个疑似改动类比对：类/方法描述符、`kotlin.Metadata`、`SourceFile`、行号/局部变量、常量字符串和路由路径。
3. 对 `ReturnData`、用户实体、配置、路径/存储类先做 JSON 与存储黑盒差分；再逐条恢复 `YueduApi` 路由的 handler 绑定。
4. 只在真实源码缺失时，以 CFR/Vineflower 的**交集**写出人工 Kotlin；每个恢复函数在提交前执行编译、`javap` ABI 对照和原 JAR 黑盒请求对照。协程应重写为 idiomatic Kotlin `suspend`，不要保留 label/Continuation 状态机。
5. 将每一个“只能按字节码得出”的改动登记为：类、原始 class SHA-256、`javap` 方法片段、功能依据、黑盒样例、回滚到原 JAR 的方法。不要用热补丁代替源码。

## 尚未验证

- 这里未声称 `work\reader-legacy-vue3` 与 JAR 同版；该目录确有同名 Kotlin 文件，是否近似必须由提交/类级 ABI/行为差分另行证明。
- 当前只做了关键八类抽样，未对全部 936 个应用 class 逐类反编译/人工审阅。
- 反编译器的样本输入未携带完整 `BOOT-INF/classes`/嵌套依赖 classpath，故 CFR 的“Could not load”列表既说明输出不可直接采用，也不能单独证明 JAR 缺依赖；JAR 本身有 85 个内嵌库。
- 尚未构建恢复版，未进行 HTTP、SSE、下载、登录、storage/data 差分；这些必须由主恢复工程的黑盒测试完成。
