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

已知限制：目前是普通 Chromium 功能基线，不是指纹浏览器；合成 Chromium 测试已覆盖
`sourceRegex` 资源 URL 捕获和 GBK HTML 输入，但尚未与原 JAR 或真实远程 WebView 做语义差分；
也尚未完成真实书源差分、ARM64、生产网络隔离、长期并发与资源预算验证。HTTP 和 SOCKS4/5
上游代理已由本地出口代理支持，并有固定 IP、
认证和凭据隔离单元测试；尚未用真实第三方代理或真实书源验证兼容率。

GitHub runner [36131976893](https://github.com/warpdotsys/reader-dev/actions/runs/36131976893)
已用真实 Chromium 验证重定向内网拦截、JS 子资源、分块 SSE、GET/POST 与 Cookie 隔离，
并在单个 Reader 镜像中完成合成书源烟测。该测试不是生产网络隔离审计：应用层出口代理
不能替代主机/容器防火墙，公网不可信书源部署仍应在网络层拦截云元数据、loopback、
RFC1918 与 IPv6 ULA。workflow 的 loopback 烟测显式启用了
`READER_BROWSER_ALLOW_PRIVATE_NETWORKS=true`，只用于固定合成夹具，不应照搬到不可信
书源可写入的生产实例。若后续兼容性回归要求回退，可在同一完整镜像中设置
`READER_APP_WEBVIEWRENDERER=remote`。不要在没有数据备份与正式发布验证的情况下直接替换
生产容器或挂载生产 `storage/data`。
