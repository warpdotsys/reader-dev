# reader-pro 3.2.14 Java/Kotlin 源码溯源报告

生成时间：2026-09-19（只读调查）  
调查对象：`work/reader-dev` 的本地 Git 对象、所有本地/已配置远端引用，以及本地 `reader-legacy-vue3` 与 `D:\Download\reader-pro-3.2.14.jar`。  
未执行：`fetch`、`checkout`、`reset`、`clean`、任何现有工程文件修改。

## 结论

推荐恢复骨架：`work/reader-legacy-vue3`，并以 Git 引用 `origin/legacy`（=`v4.0.7`）作为可追溯基线。它不是已证实的 3.2.14 源码，但有很强的同构建体系、同依赖组合、同资源基底证据：JAR 的 159 个非 class 应用资源中有 153 个与该目录逐字节一致。

不能声称它等同于 JAR 3.2.14：没有找到名为 `3.2.14` 的 tag/ref；骨架声明的项目版本是 `4.0.7`，且至少存在 5 个资源差异及 JAR 中 License 相关类等差异。恢复后应以原始 JAR 的行为和字节码为最高裁决标准。

## 仓库和引用盘点

`work` 下发现的独立 Git 仓库只有：

| 目录 | `.git` | 说明 |
| --- | --- | --- |
| `work/reader-dev` | 是 | 唯一带完整引用和历史的仓库 |
| `work/reader-legacy-vue3` | 否 | Java/Kotlin 本地源码副本 |
| `work/reader-go` | 否 | Go 重写成果，保留不动 |
| `work/reader-v5-ui-refresh` | 否 | Rust 工作副本 |
| `work/master-web-ui`、`portable-go`、`toolchains` | 否 | 非 Git 工程/工具目录 |

`reader-dev` 配置的远端：

```text
origin  https://github.com/warpdotsys/reader-dev.git (fetch/push, partial clone blob:none)
```

已存在的本地/远端分支：

```text
local:  master -> 37eff86 (v6.0.45)
        legacy-v5.2.4 -> b8c3c72
remote: origin/master -> 37eff86
        origin/legacy -> 90b62af
        origin/feat/migrate-pro-features -> b05d7e5
        origin/rust -> 3ffe008
        origin/archive/master-v5.2.4 -> b8c3c72
```

标签覆盖 `v4.0.0` 到 `v4.0.7`、`v5.0.0` 到 `v5.2.4`、`v6.0.0` 到 `v6.0.45`；没有 `3.2.14` 或相近的 `v3.*` 标签。

## 关键候选比较

| 候选 | 提交/状态 | 语言和构建 | 与 JAR 的证据 | 结论 |
| --- | --- | --- | --- | --- |
| `origin/legacy` / `v4.0.7` | `90b62afd47b08a1795365e9eecbe6da3e08af27f`，2026-08-03 | Java/Kotlin；约 142 `.kt`、104 `.java` | Spring/Kotlin/Vert.x 版本与 JAR 对齐；资源高度一致 | 首选可维护源码基线 |
| `work/reader-legacy-vue3` | 非 Git 工作目录 | 上述 legacy 线的本地副本 | `build.gradle.kts`、`ReturnData.kt` 与 `origin/legacy` 的 Git blob 一致；直接可用于新目录初始化 | 首选复制来源，但不覆盖该目录 |
| `origin/feat/migrate-pro-features` | `b05d7e54b1516ba70fd3d505299e813e09d1e067`，2026-08-01 | Java/Kotlin | 有 JAR-aligned 功能提交，但基于 `v4.0.3` | 仅作差异/缺失功能参考，早于首选候选 |
| `master` / `v5+` | `37eff86`（v6.0.45） | Rust | v5.0.0 后无 Gradle 项目构建文件 | 仅参考新前端和既有兼容分析，不作为后端恢复骨架 |

分界证据：`v5.0.0` 的 tree 已无 `build.gradle.kts`，约有 66 个 Rust 源文件且仅余 3 个 Kotlin 文件；其前置初始化提交的主题明确写为“空 master 分支，作为 Rust 重构开发分支（Kotlin 版移至 legacy）”。因此“切为默认分支”在恢复项目的语义上应指向 legacy 基线，而不是当前 Git `master`。

## 构建指纹

从 `origin/legacy:build.gradle.kts` 及本地 legacy 副本读取：

```text
group                 com.htmake
project version       4.0.7
Spring Boot plugin    2.1.6.RELEASE
Kotlin                1.5.21
Java source/target    1.8
Vert.x core/web/etc.  3.8.5
coroutines-slf4j      1.5.2
Gradle wrapper        6.1.1
application main      com.htmake.reader.ReaderUIApplicationKt
```

JAR Manifest 实测：

```text
Main-Class: org.springframework.boot.loader.JarLauncher
Start-Class: com.htmake.reader.ReaderApplicationKt
Spring-Boot-Version: 2.1.6.RELEASE
Spring-Boot-Classes: BOOT-INF/classes/
Spring-Boot-Lib: BOOT-INF/lib/
```

JAR `BOOT-INF/lib` 直接包含 `vertx-lang-kotlin-3.8.5`、`vertx-lang-kotlin-coroutines-3.8.5`、`vertx-web-3.8.5`、`kotlinx-coroutines-slf4j-1.5.2`、`kotlin-stdlib-jdk8-1.5.21`、`rhino-1.7.13-1`、`xmlpull-1.1.3.1`、`spring-boot-starter-2.1.6.RELEASE`，与上述 Gradle 基线高度吻合。

注意：JAR 的 `Start-Class` 是 `ReaderApplicationKt`，而 Gradle application 任务的 main class 配置是 `ReaderUIApplicationKt`。两者的职责差异需要在恢复工程启动验证时保留并明确处理，不能仅按 Gradle application 配置推断 JAR 的服务入口。

## JAR 资源交叉比对（已验证）

使用 .NET `System.IO.Compression.ZipFile` 读取 JAR，逐项 SHA-256 比对 `BOOT-INF/classes/` 中的非 class 资源与：

```text
work/reader-legacy-vue3/src/main/resources
```

结果：

```text
JAR non-class BOOT-INF/classes resources : 159
byte-identical with legacy resources      : 153
same relative path but different bytes    : 5
missing source path                       : 1
```

不同资源：

```text
application.yml
logback-spring.xml
epub/main.css
bookSourceDebug/index.js
bookSourceDebug/index.html
```

JAR 独有路径：

```text
META-INF/reader-pro.kotlin_module
```

JAR class 名称中还观察到 `LicenseController`、`ActiveLicense`、`License` 等相关类。该差异说明原 JAR 的实际许可/授权路径不能根据 legacy 当前代码的名称或策略擅自推断；应作为反编译和黑盒差分的优先核验项。

## 可复核命令

以下命令均为本次实际使用或等价的只读复核方式（PowerShell）：

```powershell
$r = 'C:\Users\chong\Documents\Codex\2026-09-16\g-i-t\work\reader-dev'
git -C $r remote -v
git -C $r branch -a -vv
git -C $r tag --sort=-creatordate
git -C $r for-each-ref --sort=creatordate `
  --format='%(refname:short)|%(objectname)|%(creatordate:iso-strict)|%(subject)' `
  refs/heads refs/remotes refs/tags
git -C $r show -s --format='%H%n%ad%n%s%nparents: %P' --date=iso-strict origin/legacy
git -C $r show origin/legacy:build.gradle.kts
git -C $r show origin/legacy:gradle/wrapper/gradle-wrapper.properties
git -C $r ls-tree -r --name-only origin/legacy
```

核验本地 legacy 副本的关键 blob：

```powershell
$l = 'C:\Users\chong\Documents\Codex\2026-09-16\g-i-t\work\reader-legacy-vue3'
git -C $r hash-object "$l\build.gradle.kts"
git -C $r rev-parse 'origin/legacy:build.gradle.kts'
git -C $r hash-object "$l\src\main\java\com\htmake\reader\api\ReturnData.kt"
git -C $r rev-parse 'origin/legacy:src/main/java/com/htmake/reader/api/ReturnData.kt'
```

以上两对哈希均相等；其中 `ReturnData.kt` 的 blob 为：

```text
06384337e27425afb118afcda1904c411ef67ea9
```

## 恢复边界与风险

1. Git 历史中未找到可直接确认的 3.2.14 tag/ref。仓库提交消息里存在后来的“JAR 对齐”工作记录，但这只是重写阶段对 JAR 的参考说明，不是原版源码溯源证据。
2. `origin/legacy` 的项目版本为 4.0.7，不能将较高版本号误读为必然兼容或同源码；唯一可靠判断是 JAR 字节码、资源、依赖清单和黑盒行为。
3. 153/159 资源逐字节相同和依赖版本匹配证明该候选非常接近，足够作为首阶段的可构建骨架；5 个差异资源必须由 JAR 抽取版本覆盖或经差分确认后再合并。
4. Kotlin 编译器会生成 `*Kt`、metadata 等类名，不能仅通过 Java 类名数量判断源文件差异。对于 JAR-only 或源码差异类，先用 `javap`/CFR/Vineflower 做结构还原，并人工整理为可维护 Kotlin/Java 源码。
5. 原始 JAR 必须持续保留为只读对照物；恢复成果不能通过空实现、替换数据格式或只改接口响应来“编译通过”。

## 后续建议

1. 从 `reader-legacy-vue3` 复制到新的 `work/reader-pro-restored`，不触碰现有 Go/Rust/legacy 目录。
2. 保留上述 legacy commit SHA 为新恢复工程的 `RECOVERY_BASELINE.md` 证据。
3. 先以 JAR 中的 `application.yml`、日志配置和 3 个前端/EPUB 差异资源为准建立构建与启动基线。
4. 对 `ReaderApplicationKt`、License 相关类、认证/命名空间、`/reader3` 路由优先做 `javap` 签名和黑盒 HTTP 差分。
