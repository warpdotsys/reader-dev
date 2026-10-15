# Reader Pro

从原版 Reader Pro 恢复并持续维护的 Java/Kotlin 阅读服务。后端以原始 `reader-pro-3.2.14.jar` 的行为和存储格式为兼容基线，提供 Vue 3 界面，并保留原版 Vue 2 入口。项目不再进行 Go/Rust 全量重写。

## 版本与支持范围

- **稳定版：** [v6.1.0](https://github.com/warpdotsys/reader-dev/releases/tag/v6.1.0)，默认维护分支为 `legacy`。
- **当前候选：** `v6.1.1-preview.1`，应用内部版本为 `6.1.1`。这是未完成第二阶段验收的临时版本，不是新的稳定版，不自动部署生产。
- **推荐运行形态：** Linux AMD64/ARM64 完整镜像，包含 JRE、Python、固定版本 Camoufox 和运行依赖。单独 JAR 不包含浏览器运行时，不能替代完整镜像。

请先阅读[临时版本说明与已知问题](docs/releases/v6.1.1-preview.1.md)。旧版本的测试结果不自动适用于新候选；网页被打开也不代表使用者已验收。

## 开始使用

稳定版安装参考[部署说明](deploy/reader-pro/README.md)。候选构建和下载入口见[构建与发版](docs/BUILD-AND-RELEASE.md)。测试候选时必须使用独立数据目录，不能直接挂载唯一一份正式 `storage`。

完整镜像默认使用 Vue 3；`READER_APP_WEBUI=vue2` 可切回原版界面。普通 JAR 的默认入口仍是原版界面，启动时使用 `--reader.app.webUi=vue3` 可选择已打包的 Vue 3。

本机运行须同时设置回环监听和新的工作目录；仅换端口并不能隔离存储。不要把管理密码、访问令牌或许可证私钥写进仓库、公开命令示例或问题反馈。

## 开发与验证

使用 JDK 11、仓库 Gradle Wrapper 和 Node.js 24。Node 依赖由锁文件安装。下列命令应在**新建的独立检出目录**运行；不要在包含私人测试输入的旧 `build/` 目录执行 `clean`。

```bash
npm ci --prefix web-vue3
npm test --prefix web-vue3
npm run build --prefix web-vue3
./gradlew -PreaderWebUi=vue3 test bootJar --max-workers=2 --no-build-cache --no-daemon
```

Windows 使用 `gradlew.bat`。仅构建旧界面的本机安全入口为 `pwsh -NoProfile -File ./scripts/build.ps1 -JavaHome <JDK11目录>`；该脚本不会清空既有 `build/`。完整镜像、浏览器契约和可重复构建由 GitHub 托管 runner 验证。

## 功能与限制

- 书源、书架、分组及 TXT/EPUB/PDF 阅读沿用恢复后的业务实现；第三方书源是否可用取决于站点规则、网络与授权状态。
- 候选增加独立 OPDS 鉴权、目录及本人书架内本地文件下载。Windows 原生运行若缺少安全目录句柄会拒绝下载，不能通过放松路径检查解决。
- [许可证中心](license-server/README.md)支持独立部署、SMTP 验证码和试用申领。测试邮件送达不代表生产邮箱投递成功。
- 新候选的完整备份恢复与自启工具仍为实验性开发，不能作为生产运维入口。详见[维护工具状态](scripts/local-reader/README.md)。

## 文档入口

- [构建与发版](docs/BUILD-AND-RELEASE.md)：Actions 分工、制品身份、候选草稿与稳定发布。
- [候选版本说明](docs/releases/v6.1.1-preview.1.md)：变更、已知问题、验收范围及升级回滚。
- [第二阶段状态](docs/SECOND-STAGE-ACCEPTANCE-2026-10-15.md)：已完成层次和暂停时仍缺少的验收。
- [v6.1.0 交付记录](docs/DELIVERY-V6.1.0-2026-10-15.md)：上一稳定版历史，不能视为当前生产状态证明。
- [维护路线图](docs/ROADMAP.md)和[Vue 3 迁移核查](docs/VUE3-UI-MIGRATION.md)：设计与兼容性背景。

原始 JAR、只读备份与历史差分报告保留各自证据身份。不会以空实现、反编译伪代码或跳过的测试宣称功能完成。
