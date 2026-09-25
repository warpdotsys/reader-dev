# Production deployment

`reader-pro-restored` runs beside the existing Reader container and binds only
to `127.0.0.1:18088`. The host Nginx terminates TLS for `read.medwarp.cn`.

Required server-side files that are deliberately not committed:

- `.env`: authentication and deployment configuration, mode 600;
- `storage/`: imported Reader data, owned by UID/GID 10001;
- `logs/`: application logs, owned by UID/GID 10001;
- `reader-4.0.7.jar`: output of `scripts/build.ps1`.

## 正式发布

正式版本通过 `.github/workflows/release.yml` 发布。准备与标签同名的
`docs/releases/<tag>.md` 后，从 `legacy` 上创建 `v*-restored.*` 标签。流水线会：

1. 在 GitHub 托管 runner 上重新运行测试并构建 JAR；
2. 生成 SHA-256 并上传不可变构建制品；
3. 部署到 production 环境，执行健康检查、UTF-8 与公网烟雾测试；
4. 失败时恢复上一版 JAR、Docker 镜像和 Nginx 配置；
5. 仅在部署成功后创建带已知问题说明的 GitHub prerelease。

以下命令只用于紧急手动恢复，不作为正常发版流程：

```bash
cd /opt/reader-pro-restored
docker compose config --quiet
docker compose build
docker compose up -d
curl -fsS http://127.0.0.1:18088/
```

## 实验性内置浏览器镜像

`Dockerfile.browser` 将版本匹配的 Playwright Chromium、系统库与浏览器版
Reader JAR 放进**同一个容器**；运行时设置 `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`
和 `READER_APP_WEBVIEWRENDERER=local`，不使用宿主机 Chrome 或第二个 WebView
容器。它不替换上面的生产 `Dockerfile` 或现有 `docker compose` 部署。

在 JDK 11 环境下，从仓库根目录构建两个独立产物：

```bash
PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 ./gradlew clean test bootJar bootJarBrowser --no-daemon
cp build/libs/reader-4.0.7-browser.jar deploy/reader-pro/
docker build -f deploy/reader-pro/Dockerfile.browser -t reader-browser:smoke deploy/reader-pro
```

`reader-4.0.7.jar` 是不含 Playwright 的普通版；
`reader-4.0.7-browser.jar` 才包含 Java 驱动。构建后的 JAR 复制到镜像上下文
只是构建输入，不应提交到 Git。GitHub 托管 runner 的
`.github/workflows/browser-image.yml` 负责构建镜像并在隔离数据目录下运行合成书源
搜索及 Cookie 回归；在该工作流实际通过前，不将镜像视为已验收。

已知限制：目前只是普通 Chromium 功能基线，不是指纹浏览器；
`sourceRegex` 与非 UTF-8 `encode` 在本地模式下显式报错；真实书源、代理、
多用户并发、容器沙箱和资源预算仍需验证。若测试不通过，继续使用默认
`READER_APP_WEBVIEWRENDERER=remote` 和现有普通镜像。不要把实验镜像直接
替换生产容器或挂载生产 `storage/data`。
