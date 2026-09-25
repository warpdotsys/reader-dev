# 内置浏览器功能基线 PoC

此目录是独立的 Playwright Java + Chromium 功能探针，**未接入 Reader 主程序，也不是指纹浏览器**。本机测试使用已有 Chrome；`Dockerfile` 则从与 Java 依赖同版本的 Playwright 镜像取得浏览器及系统依赖，把它们打包进 PoC 容器，运行时不依赖宿主机 Chrome，也不下载浏览器。该容器尚未构建验收，更不是 Reader 正式镜像。当前只验证 HTTP GET/POST、请求头、页面脚本、按命名空间隔离的内存 Cookie 和超时。`sourceRegex`、字符编码、远程 CookieStore 持久化、多用户并发、浏览器进程监管、SSRF、真实书源及指纹能力都未覆盖。

Windows 本机运行合成页面测试（`READER_BROWSER_EXECUTABLE` 必须指向已有浏览器，否则浏览器测试会跳过）：

```powershell
$env:JAVA_HOME = (Resolve-Path '..\.tools\jdk-11.0.8').Path
$env:READER_BROWSER_EXECUTABLE = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
$env:PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = '1'
..\gradlew.bat -p . test --no-daemon
```

只允许用本机合成页面或自己有权访问的站点做探针。`BrowserProbe` 的 CLI 会输出页面 HTML，可能含敏感内容，切勿对生产账号或私有书源直接运行并共享日志。Playwright 与任意系统 Chrome 的组合不受上游兼容性保证，因此正式构建不得复用此路径。Gradle 的 `test`/`run` 任务和 Java 启动器均显式设置跳过浏览器下载；从其他入口直接调用 Playwright 时仍须设置 `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`。

容器打包与离线页面 smoke（需要 Docker；此机器尚未执行）：

```powershell
docker build -f browser-poc/Dockerfile -t reader-browser-poc:local .
docker run --rm --init --read-only --tmpfs /tmp:rw,nosuid,size=256m reader-browser-poc:local
```

上面的 `Dockerfile` 使用官方测试/开发镜像来验证浏览器随镜像交付，并非访问不可信书源的生产安全配置。正式 Reader 单容器还需锁定基础镜像 digest、缩减镜像、配置非 root 沙箱/seccomp、限制内网访问、实测进程与资源上限，再与 Java/Kotlin 服务共进程树部署。不能仅将这个 PoC 镜像与 Reader 镜像并排部署后称为“内置”。
