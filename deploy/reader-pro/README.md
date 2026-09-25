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

1. 在 GitHub 托管 runner 上重新运行测试并构建唯一完整 JAR 和包含 Chromium 的镜像；
2. 为 JAR、镜像归档生成 SHA-256 并上传不可变构建制品；
3. 服务器校验散列后加载已构建镜像，不在生产机编译；执行健康检查、UTF-8 与公网烟雾测试；
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

## 唯一的完整镜像

`Dockerfile` 将版本匹配的 Playwright Chromium、系统库与 Reader JAR
放进**同一个容器**；运行时设置 `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`
和 `READER_APP_WEBVIEWRENDERER=local`，不使用宿主机 Chrome 或第二个 WebView
容器。只构建一个完整版本，不再构建或发布轻量版。远程 WebView 配置暂留作回滚。

在 JDK 11 环境下，从仓库根目录构建唯一产物：

```bash
PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 ./gradlew clean test bootJar --no-daemon
cp build/libs/reader-4.0.7.jar deploy/reader-pro/
docker build -f deploy/reader-pro/Dockerfile -t reader-browser:smoke deploy/reader-pro
```

`reader-4.0.7.jar` 包含 Java 浏览器驱动；Chromium 二进制在镜像内，
不是 JAR 内。构建后的 JAR 复制到镜像上下文只是构建输入，不应提交到 Git。
GitHub 托管 runner 的
`.github/workflows/browser-image.yml` 负责构建镜像并在隔离数据目录下运行合成书源
搜索及 Cookie 回归；生产部署仍需另行验证和发布。

已知限制：目前只是普通 Chromium 功能基线，不是指纹浏览器；
`sourceRegex` 与非 UTF-8 `encode` 在本地模式下显式报错；真实书源、代理、
多用户并发、容器沙箱和资源预算仍需验证。本地浏览器默认执行应用层内网目标检查，
但它不能替代容器/主机出口防火墙；DNS rebinding、代理端二次解析和非 HTTP 通道
仍需在部署网络单独限制。Playwright 路由处理重定向链时只检查首个 URL，
因此重定向后的私网目标必须由出口网络策略拦截。仅受控测试/自管内网可显式设置
`READER_BROWSER_ALLOW_PRIVATE_NETWORKS=true` 放行私网目标。若切换后有兼容问题，可在同一
完整镜像中设置 `READER_APP_WEBVIEWRENDERER=remote` 回退。不要在没有
数据备份与正式发布验证的情况下直接替换生产容器或挂载生产 `storage/data`。
