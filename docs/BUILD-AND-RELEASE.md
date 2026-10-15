# Reader Pro 构建与发版

所有发布构建使用 GitHub 托管 runner，不依赖个人电脑或自托管 runner。软件发布与生产部署分开；预发布永远不部署生产。

## Actions 分工

| 工作流 | 用途 | 允许的外部变更 |
| --- | --- | --- |
| Java Kotlin CI | 先检查源码和发布约束，再并行构建 Reader、许可证服务及 Windows 生成测试；双 clean 校验 JAR 字节一致 | 仅上传 Actions 制品 |
| Vue 3 preview build | 前端构建与生成账号的真实浏览器场景 | 仅上传测试证据 |
| Native release artifact rehearsal | 同一 JAR 打入 AMD64/ARM64 完整镜像，执行浏览器、重载及发布导入验证 | 仅上传软件与证据，不登录镜像仓库 |
| Prepare Reader preview | 复用上述工作流，比对跨任务 JAR 和镜像哈希，整理临时发行包 | 默认只上传候选；手动选择后可创建草稿预发布 |
| Stable release Reader Pro | 稳定 SemVer 标签的正式镜像与 Release 发布 | 允许正式发布；生产部署另受显式开关控制 |

源码检查失败后，不继续昂贵的预发布镜像工作。制品按源码 SHA 命名，只消费本次运行的产物；下载摘要不匹配即失败。第三方 Actions 固定到提交 SHA。恢复生产等历史工作流只供明确授权的运维使用，不属于临时发布路径。

## 本地开发

使用 JDK 11、Node.js 24 和随仓库提供的 Gradle Wrapper。不要把用户数据放进源码检出目录，也不要在承载私人测试输入的旧 `build/` 下执行 clean。

```bash
npm ci --prefix web-vue3
npm test --prefix web-vue3
npm run build --prefix web-vue3
./gradlew -PreaderWebUi=vue3 test bootJar --max-workers=2 --no-build-cache --no-daemon
```

Windows 将 `./gradlew` 替换为 `./gradlew.bat`。许可证服务独立构建：进入 `license-server` 后运行 `./gradlew test fatJar --max-workers=2 --no-daemon`。仅本机 JAR 成功不能代替完整镜像验收。

## 临时版本准备

1. 在 `release/**` 分支统一更新应用、CLI、两套前端和根锁文件的基础版本；修改 `preview.json`，例如基础版本 `6.1.1`、候选标签 `v6.1.1-preview.1`。
2. 编写对应 `docs/releases/<候选标签>.md`，必须包含已知问题、验证范围、升级与回滚。
3. 推送分支，`Prepare Reader preview` 自动运行。仅当源码、UI、双架构与导入验证均成功时，生成 `reader-preview-<完整SHA>`；候选制品保留 7 天，过期后重新构建，不借用其他提交的旧产物。
4. 下载候选后，在其目录运行 `sha256sum -c SHA256SUMS`，检查 `PREVIEW.json`。包内包含 Reader JAR、独立许可证 JAR、两个架构的完整镜像归档、回执和发行说明。
5. 需要持久保留时，在 Actions 手动运行相同分支，选择 `create_draft=true`。工作流会重新通过全部门槛，再创建 **Draft + Pre-release + 非 Latest** 的新草稿。已有标签或 Release 会被拒绝，不能用重跑覆盖。

发布草稿是单独的人工作业；当前准备任务不会自动把草稿转成公开发行。上传中断可能留下不完整草稿，先核对附件与清单，不盲目重跑或删除。

## 稳定版与生产

稳定标签只接受 `vX.Y.Z`，预发布标签被排除。正式版继续执行现有原生镜像、双仓库和回滚约束，不能用本地手工包替换已验证制品。

正式发布默认不执行 SSH 部署；只有仓库变量 `READER_ENABLE_PRODUCTION_DEPLOY=true` 才允许进入原有生产环境步骤。当前任务不设置此变量、不连接服务器。启用前仍需明确授权并复核生产备份与环境审批。`release-resume.yml` 是旧正式发布恢复工具，不用于本候选。

## 失败处理

- 源码或测试失败：保存日志，修改后以新提交重新运行。不要用 `continue-on-error` 放行。
- 双 clean、JAR 或镜像 SHA 不同：拒绝组包，调查构建输入；不通过归一化 ZIP 内容掩盖差异。
- 浏览器测试跳过：普通 JVM 的跳过不算执行；原生任务要求两套共 35 项实际运行并保存 XML。
- 部分草稿附件缺失：保留现场并核对任务和附件，不移动标签，不覆盖稳定版。

GitHub 官方说明：[复用工作流](https://docs.github.com/en/actions/how-tos/reuse-automations/reuse-workflows)及[创建草稿与预发布](https://cli.github.com/manual/gh_release_create)。
