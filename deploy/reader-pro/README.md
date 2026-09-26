# Production deployment

`reader-pro-restored` runs beside the existing Reader container and binds only
to `127.0.0.1:18088`. The host Nginx terminates TLS for `read.medwarp.cn`.

Required server-side files that are deliberately not committed:

- `.env`: authentication and deployment configuration, mode 600;
- `storage/`: imported Reader data, owned by UID/GID 10001;
- `logs/`: application logs, owned by UID/GID 10001;
- release image: pulled by digest from the registry; production never rebuilds
  from a JAR on the host.

## 正式发布

正式版本通过 `.github/workflows/release.yml` 发布。全部验收完成后，准备与标签同名的
`docs/releases/<tag>.md`，并从 `legacy` 上创建稳定 SemVer 标签（例如 `v4.1.0`）。
触发前必须同步提升 `build.gradle.kts`、`web/package.json` 与
`web-vue3/package.json` 的版本号；流水线拒绝版本不一致、非稳定 SemVer、非 `legacy`
祖先提交，或不高于当时最新 GitHub 正式 Release 的标签。

发布前还必须在仓库设置中配置以下值，工作流不会猜测仓库名或凭据：

- Variables：`GHCR_IMAGE`（格式 `ghcr.io/namespace/image`）与
  `DOCKERHUB_IMAGE`（格式 `namespace/image`）；
- Secrets：`DOCKERHUB_USERNAME`、`DOCKERHUB_PASSWORD`；
- 已有生产连接 Secrets：`PROD_SSH_HOST`、`PROD_SSH_USER`、`PROD_SSH_KEY`、
  `PROD_SSH_KNOWN_HOSTS`。其中 known_hosts 必须是 `cdn.medwarp.cn` 的已验证主机键。

`PROD_SSH_HOST` 必须精确为 `cdn.medwarp.cn`，工作流会拒绝其他目标。若 Docker Hub
镜像为私有镜像，或服务器可能被匿名拉取限流，部署账户必须已在服务器完成可用的
`docker login`；工作流在停止旧容器前先拉取并按 digest 验证新镜像，认证或限流失败不会
中断正在运行的服务。生产服务器不会接收 Docker Hub token。

SSH 只使用仓库 Secret 中的已验证 `known_hosts`，并显式启用
`diffie-hellman-group14-sha256` 与 `IdentitiesOnly=yes`；这与该主机已验证的兼容
协商保持一致。每份正式发布说明还必须有 `## 已知问题…` 章节，缺失时不会开始构建。

`requirements-camoufox.lock` 已按 Ubuntu 22.04 / Python 3.10 为 38 个固定依赖记录
52 个兼容 amd64/arm64 wheel 的 PyPI SHA-256，且本机校验了依赖闭合和 pip 锁文件解析。
这尚不等于镜像安装成功：正式流水线会强制 `pip --require-hashes`，并从
`base-images.lock` 读取已核验的 Temurin 与 Playwright manifest digest，再以这些
digest 构建；发布时不会重新从浮动 tag 解析 digest。Dockerfile 不接受浮动基础镜像
默认值，并会移除基础镜像遗留的 apt 源；系统包只使用 `apt-sources.list` 所列的 Ubuntu
快照。更新任一锁文件必须作为可审查的发布链路变更。

流水线会：

1. 在 GitHub 托管 runner 上重新运行 Vue 3、Java/Kotlin 测试，构建可校验 JAR；
2. 构建包含 Camoufox 的唯一完整镜像并在镜像内进行健康和浏览器运行时烟测；
3. 将同一镜像按版本标签推送到 GHCR 和 Docker Hub，读取并比对两个 registry digest；
4. 创建正式 GitHub Release，附 JAR、SHA-256 和镜像 digest 清单；
5. 由 `cdn.medwarp.cn` 拉取 Docker Hub 的 digest 引用，不从本地 tar 或服务器源码构建；
   激活前要求可用空间至少为 `3 × storage + 2 GiB`，保存 storage、配置和旧镜像回滚快照，
   再做健康与 `read.medwarp.cn` 公网烟测。公网探测同时校验 `/assets/reader-release.json`
   中的版本和构建提交；失败会先确认容器正使用本次 digest，再自动恢复已验证的旧镜像与数据。

可在创建标签前执行以下本地静态检查；它不访问 registry 或生产服务器：

```bash
node scripts/check-release-pipeline.mjs
```

以下命令只用于已有 digest 镜像的紧急手动恢复，不作为正常发版流程：

```bash
cd /opt/reader-pro-restored
READER_IMAGE='registry/image@sha256:...' docker compose config --quiet
READER_IMAGE='registry/image@sha256:...' docker compose up -d --no-build
curl -fsS http://127.0.0.1:18088/
```

## 唯一的完整镜像

`Dockerfile` 将 Reader JAR、Python 解释器、固定版本的 Camoufox `0.5.6` /
Playwright `1.62.0`、Firefox 依赖和经 SHA-256 校验的
`v152.0.4-beta.30` 浏览器放进**同一个容器**。运行时设置
`PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`、
`READER_APP_WEBVIEWRENDERER=camoufox`，并使用
`/opt/reader-camoufox/bin/python` 启动 worker；不要求 Docker 宿主机安装
Chrome、Firefox 或独立 WebView 容器。只构建一个完整版本，不再构建或发布轻量版。
远程 WebView 配置暂留作回滚。

在 JDK 11 环境下，从仓库根目录构建唯一产物：

```bash
PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 npm ci --prefix web-vue3
npm test --prefix web-vue3
npm run build --prefix web-vue3
PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 ./gradlew -PreaderWebUi=vue3 clean test bootJar --no-daemon
# 用当前版本 JAR 创建隔离镜像上下文，再传入 --build-arg READER_JAR=<jar filename>
```

版本化 Reader JAR 包含 Java 端 renderer 和 Python worker 脚本；Camoufox
二进制及其 Python 运行时在镜像内，不在 JAR 内。构建后的 JAR 复制到镜像上下文只是
构建输入，不应提交到 Git。GitHub 托管 runner 的
`.github/workflows/browser-image.yml` 会先安装固定浏览器并执行 GET、POST、脚本、
Cookie 用户命名空间和 `sourceRegex` 资源捕获测试，再构建镜像并在隔离数据目录下运行
合成书源搜索及 Cookie 回归；生产部署仍需另行验证和发布。

已知限制：Camoufox 仍处于候选接入阶段。本机 Windows 隔离运行时已用固定浏览器验证
GET/POST、脚本子资源、Cookie 回写与 `sourceRegex` 捕获，并完成 Gradle/JAR 构建；
但 Linux 容器构建和镜像内真实渲染尚未通过 GitHub 托管 runner 验收，不能把旧 Chromium
runner 或 Windows 结果视为生产镜像验收。尚未完成原 JAR 与远程
WebView 的完整语义差分、真实书源差分、ARM64、生产网络隔离、长期并发与资源预算验证。
HTTP 和 SOCKS4/5 上游代理由本地出口代理支持，并有固定 IP、认证和凭据隔离单元测试；
尚未用真实第三方代理或真实书源验证兼容率。

GitHub runner [36131976893](https://github.com/warpdotsys/reader-dev/actions/runs/36131976893)
记录的是旧 Chromium 实现的功能基线：重定向内网拦截、JS 子资源、分块 SSE、GET/POST 与
Cookie 隔离，以及单个 Reader 镜像内的合成书源烟测。Camoufox 的对应验证会由新的
`browser-image.yml` job 重新执行。应用层出口代理不能替代主机或容器防火墙；公网不可信
书源部署仍应在网络层拦截云元数据、loopback、RFC1918 与 IPv6 ULA。workflow 的 loopback
烟测显式启用了 `READER_BROWSER_ALLOW_PRIVATE_NETWORKS=true`，只用于固定合成夹具，
不应照搬到不可信书源可写入的生产实例。若后续兼容性回归要求回退，可在同一完整镜像中设置
`READER_APP_WEBVIEWRENDERER=remote`。不要在没有数据备份与正式发布验证的情况下直接替换
生产容器或挂载生产 `storage/data`。
