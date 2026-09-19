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
