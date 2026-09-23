# reader-pro 3.2.14 Java/Kotlin 恢复：第一阶段报告

日期：2026-09-19（Asia/Shanghai）

本文件是第一阶段的历史快照，不能代表当前完成状态。后续已恢复许可证中心并扩展认证、文件和本地阅读差分；以 `LICENSE-CENTER-RECOVERY.md`、`AUTH-LIFECYCLE-DIFF.md`、`FILE-LIFECYCLE-DIFF.md`、`LOCAL-READING-DIFF.md` 及最新生产部署记录为准。

## 结论

第一阶段已经产生可编译、可启动的独立 Java/Kotlin 恢复工程，而不是方案或空脚手架。工程固定以 Git 提交 `90b62afd47b08a1795365e9eecbe6da3e08af27f`（`origin/legacy` / `v4.0.7`）为真实源码骨架，再用原始 JAR 的 158 个非 class 应用资源覆盖资源层。原始 Web 前端因此保持 JAR 版本。

恢复目录本身已初始化为独立 Git 工作树，当前分支为 `restore/reader-pro-3.2.14`，HEAD 锚定上述 legacy 提交；所有恢复改动都显示为相对该提交的明确 diff。尚未提交或推送恢复改动，避免把第一阶段未核销功能误当成正式完成品。

当前成果已经达到：

- GitHub 仓库 `warpdotsys/reader-dev` 的默认分支已从 Rust `master` 切换为 Java/Kotlin `legacy`，并已通过 GitHub API 回读确认；本地 `origin/HEAD` 也已指向 `origin/legacy`；
- `clean bootJar` 成功；
- 生成包的 Spring Boot 入口与原 JAR 一致，为 `com.htmake.reader.ReaderApplicationKt`；
- 首页、`/reader3/getSystemInfo`、`/reader3/getTxtTocRules` 实际启动探针通过；
- 生成包的 85 个 `BOOT-INF/lib` 文件名与原 JAR 逐项完全一致；
- 使用真实 `storage/data` 的隔离元数据快照，原 JAR 与恢复版均能通过已有 `accessToken` 读取同一用户命名空间下的 169 条书架记录和 429 条简化书源记录；
- 没有修改 `work/reader-dev`、`work/reader-go`、`work/reader-legacy-vue3`，也没有写回 `D:\Download\storage`。

这还不是“完整恢复”。许可证模块的 14 条路由、若干 Pro 专有业务路径、SSE、下载和完整 storage 内容差分仍待恢复或验证。

## 原始 JAR 保全

| 项目 | 已从 JAR 验证 |
| --- | --- |
| 原文件 | `D:\Download\reader-pro-3.2.14.jar` |
| 原文件大小 | 72,913,887 bytes |
| SHA-256 | `B26FB4769D689D98FF26408CE79A275D719F360906C84ACF52FF404E98030C8C` |
| 只读备份 | `reference/original/reader-pro-3.2.14.original.jar` |
| 备份属性 | `ReadOnly, Archive` |

原文件未被覆盖。备份散列与原文件一致，并记录在 `reference/original/SHA256SUMS.txt`。

## JAR 与构建版本

### 已从 JAR 验证

- Spring Boot 可执行 fat JAR；`Main-Class=org.springframework.boot.loader.JarLauncher`。
- `Start-Class=com.htmake.reader.ReaderApplicationKt`。
- Spring Boot `2.1.6.RELEASE`、Spring Framework `5.1.8.RELEASE`。
- 应用 class 共 936 个，全部为 class major 52（Java 8）。
- 754 个 class 带 Kotlin Metadata；936 个有 `SourceFile`；755 个有 `LineNumberTable`。
- Kotlin runtime `1.5.21`、coroutines `1.5.2`、Vert.x `3.8.5`。
- `BOOT-INF/lib` 有 85 个 JAR；恢复包现已与这 85 个文件名逐项一致，包括 Netty `4.1.42.Final`。
- 原 JAR 含完整 `web`、`simple-web`、EPUB、DTD、默认数据和书源调试资源。

### 已从近似源码推断并经构建验证

- Gradle Wrapper `6.1.1`、Kotlin Gradle plugin `1.5.21`、Java target 8。
- 恢复构建使用经发布方 SHA-256 验证的便携 AdoptOpenJDK `11.0.8+10`；JAR 目标仍为 Java 8。
- 为匹配原 JAR，`bootJar` 明确使用无 UI 的 `ReaderApplicationKt`，排除仅供桌面打包使用的 JavaFX 模块，并将 Netty 锁定为 `4.1.42.Final`。

## 最接近源码的证据

本地唯一 Git 仓库为 `work/reader-dev`。所有本地和远端引用中：

- `origin/legacy` 与 tag `v4.0.7` 同指向提交 `90b62afd47b08a1795365e9eecbe6da3e08af27f`；
- `master` 自 `v5.0.0` 后是 Rust 线，不适合作为 Java/Kotlin 恢复骨架；
- `origin/feat/migrate-pro-features` 基于较早的 `v4.0.3`，包含许可证和 JAR 对齐工作，可作为缺失 Pro 功能的二级来源，但不能整体覆盖较新的 legacy 骨架；
- 没有任何 tag/ref 对应 `3.2.14`。

JAR 与 legacy 骨架的强关联证据：

- Spring Boot、Kotlin、Vert.x、coroutines、Rhino、xmlpull 版本组合一致；
- JAR 的 159 个非 class 应用资源中，153 个与骨架逐字节相同；
- 5 个不同资源及 JAR 多出的 Kotlin module 已单独识别；恢复工程直接采用 JAR 原资源，避免前端和配置漂移。

因此该提交是“极接近、同构建体系的真实源码骨架”，但不是已证明同版的 3.2.14 源码。

按项目目标，远端默认分支已实际切换为 `legacy`。此操作只改变 GitHub 默认分支指针和本地 `origin/HEAD`，没有 checkout、reset、clean 或覆盖任何工作树文件。

## 可重复构建

在恢复工程根目录执行：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build.ps1
```

底层等价命令：

```powershell
$env:JAVA_HOME = (Resolve-Path '.\.tools\jdk-11.0.8').Path
$env:PATH = "$env:JAVA_HOME\bin;$env:PATH"
.\gradlew.bat clean bootJar --no-daemon
```

本阶段最后一次实际结果为 `BUILD SUCCESSFUL`。产物为 `build/libs/reader-4.0.7.jar`；保留 4.0.7 文件名是为了避免把近似骨架误标成已经完全恢复的 3.2.14。

## 首轮黑盒差分

### 无登录、隔离工作目录

原 JAR 端口 18080，恢复版端口 18081；两端使用各自独立的 workDir。

| 探针 | HTTP | ReturnData / 数据 | 原始响应字节 |
| --- | --- | --- | --- |
| `/` | 两端 200，5625 bytes | 原 JAR 前端首页 | 完全一致 |
| `/reader3/getSystemInfo` | 两端 200 | `isSuccess=true`，字段一致 | 不一致，仅运行时 `freeMemory` 不同 |
| `/reader3/getUserInfo` | 两端 200 | `isSuccess=true`，结构一致 | 完全一致 |
| `/reader3/getUserConfig` | 两端 200 | `isSuccess=false`，`errorMsg=没有备份文件` | 完全一致 |
| `/reader3/getBookshelf` | 两端 200 | `isSuccess=true`，空数组 | 完全一致 |
| `/reader3/getBookGroups` | 两端 200 | 均 5 项、字段一致 | 不一致；原 JAR 固有中文乱码，近似源码为正常中文 |
| `/reader3/getTxtTocRules` | 两端 200 | `isSuccess=true`，18 项 | 完全一致 |
| `/reader3/getRssSources` | 两端 200 | `isSuccess=true`，空数组 | 完全一致 |
| `/reader3/httpTTS/list` | 两端 200 | `isSuccess=true`，空数组 | 完全一致 |
| 非法用户名登录 | 两端 200 | `isSuccess=false`，错误文案一致 | 完全一致 |

合计：10 项中 8 项原始响应字节完全一致；另外 2 项差异均已定位，不是未知漂移。

### accessToken、用户命名空间与 storage 快照

从 `D:\Download\storage\data` 仅读取并复制 51 个顶层/用户元数据文件（61,188,965 bytes）到两套隔离 workDir，没有复制缓存书籍正文，也没有写回原目录。两端以 `reader.app.secure=true` 启动，使用快照中已有 token 测试：

| 探针 | 原 JAR | 恢复版 | 结论 |
| --- | ---: | ---: | --- |
| `getUserInfo` | 200 / success | 200 / success | 字段集合一致；登录时间为动态值 |
| `getUserConfig` | 200 / success | 200 / success | 顶层与嵌套字段集合一致；字符串编码值有差异 |
| `getBookshelf` | 169 项 | 169 项 | 每项字段集合一致；字符串编码值有差异 |
| `getBookGroups` | 5 项 | 5 项 | 字段一致；同上，原 JAR 返回乱码文本 |
| `getBookSources?simple=1` | 429 项 | 429 项 | 原始响应字节完全一致 |

已成功证明恢复版能识别原 storage 的用户文件、accessToken 和用户命名空间。尚未证明完整 2.7 GB storage（书籍缓存、WebDAV、本地书、备份目录）全部兼容。

### 非 JSON 响应

- 首页静态 HTML 已做字节级对比并完全一致。
- SSE、文件下载、TTS 音频、封面和导出响应尚未进行终止帧、header、流长度和断流差分；不得视为已恢复。

## 可恢复程度与主要风险

### 高可恢复

- 构建工具链、依赖、启动入口、静态资源、配置、DTO、ReturnData、绝大多数既有 Java/Kotlin 业务源码。
- JAR 保留 Kotlin Metadata、源文件名和大部分行号，可用 `javap` 对人工恢复代码做 ABI 校验。

### 中高风险

- Kotlin suspend/coroutine、默认参数、内联泛型和大型路由 lambda。CFR 0.152 与 Vineflower 1.10.1 的抽样均出现不可编译伪代码或 `Couldn't be decompiled`，只能作为证据，不能直接放进 `src`。
- 原 JAR 中有 936 个应用 class，当前恢复构建有 915 个；集合比较为原 JAR 独有 58 个、恢复版独有 37 个，其中部分只是编译器合成类命名变化，但不能全部按合成差异忽略。

### 已确认缺口

- `LicenseController`、`License`、`ActiveLicense` 及 14 条许可证路由尚未合入当前恢复工程；`origin/feat/migrate-pro-features` 有人工维护源码候选，需要逐方法与 JAR 校验后再移植。
- 原 JAR 的 `YueduApi` 有 `initRouter$125` 至 `$138`，当前骨架没有对应合成 handler，主要与许可证路由相关。
- BookController 的部分 Pro 专有路径仍需逐方法 ABI/黑盒核销；类名相同不代表方法体同版。
- 依赖文件名已完全一致，但尚未对 85 个嵌套依赖逐个内容散列比较；同文件名不等于字节相同。
- 没有做完整 storage、SSE、下载、远程书源、WebDAV、MongoDB、EPUB/TXT 导出或许可证行为验证。

工程没有为了编译引入新的空业务实现。骨架中 `DB` 基类的空虚方法与原 JAR 字节码一致，具体 JSON/SQL 实现存在；这是 JAR 已验证的原设计，不是本次伪造的占位。

## 证据文件

- `JAR-STRUCTURE-AND-VERSION.md`：JAR、依赖、字节码、资源和配置取证。
- `GIT-PROVENANCE.md`：分支、标签、提交、资源相似度和候选骨架证据。
- `DECOMPILATION-RISK.md`：CFR/Vineflower/javap 抽样与 Kotlin 恢复风险。
- `scripts/compare-running-instances.ps1`：可重复的无状态首轮差分脚本。

## 下一阶段优先级

1. 将 `origin/feat/migrate-pro-features` 的许可证真实源码作为候选，逐方法用 JAR 的 ABI、常量、路由和黑盒响应核对后移植，不能整分支覆盖。
2. 生成原 JAR与恢复版的公开方法/路由矩阵，先核销 Pro 独有 BookController、UserController、FileController、HttpTTSController 路径。
3. 扩展差分到有效登录、token 续期/登出、完整用户命名空间、文件路径和写后格式；所有写测试继续使用隔离副本。
4. 对 SSE/下载/导出建立流式 harness，要求终止帧、Content-Type、Content-Disposition、长度与断流行为一致。
5. 完成后再考虑 Vue 3；当前仍固定使用 JAR 原版前端。
