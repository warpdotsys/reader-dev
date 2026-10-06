# reader-pro-restored

本目录用于从 `reader-pro-3.2.14.jar` 恢复可维护、可重复构建的 Java/Kotlin 混合工程。

## 证据边界

- `reference/original/reader-pro-3.2.14.original.jar`：原始 JAR 的只读备份，不参与写入或原地修改。
- `reference/original/SHA256SUMS.txt`：原始 JAR 的 SHA-256 记录。
- `reports/`：结构、版本、源码溯源、恢复风险及差分报告。
- `src/` 与构建文件：仅在确认最接近的真实源码骨架后建立。

状态术语：

- **已从 JAR 验证**：由归档内容、字节码、资源或实际运行直接确认。
- **已从近似源码推断**：来自 Git 历史或相似源码，尚未证明与 JAR 同版。
- **已成功重建**：恢复工程本地编译或运行测试已实际通过。
- **尚未验证**：没有足够证据或测试覆盖。

不得覆盖原始 JAR，不得以空实现替代缺失业务逻辑。

## 当前开发主线

内置浏览器、参考 Rust 分支设计语言的 Vue 3 界面、项目 Markdown 文档是当前主要工作。候选单容器镜像内置 Camoufox，Vue 3 可通过 `-PreaderWebUi=vue3` 打包并以 `--reader.app.web-ui=vue3` 选择；普通 JAR 仍默认原版 Vue 2 和远程渲染配置，线上也尚未切换。原始 JAR 的行为和数据格式仍是后端兼容基线。实施顺序、验收门槛和已知边界见 [维护路线图](docs/ROADMAP.md)与 [Vue 3 UI 迁移核查](docs/VUE3-UI-MIGRATION.md)。

浏览器的独立功能探针见 [browser-poc](browser-poc/README.md)。项目维护一个完整 JAR 和一个正式镜像定义：JAR 包含 Java renderer 与 Python worker，固定版本的 Camoufox 浏览器、Python/JRE 运行时及系统依赖由镜像携带，不要求宿主机 Chrome 或独立 WebView 容器。最新源码将完整镜像默认界面改为 Vue 3，并保留 `READER_APP_WEBUI=vue2` 回退；普通 JAR 默认值不变，新的镜像门禁仍须独立验收，详见[默认 Vue 3 候选增量](docs/DEFAULT-VUE3-IMAGE-2026-10-06.md)。现有远程渲染配置暂留作浏览器回滚。此前 `08cccfca` 自己的[正常门禁](docs/CAMOUFOX-SOURCE-DOCUMENT-2026-10-06.md)和[双架构离线长测](docs/BROWSER-SOAK-2026-10-06.md)已通过，不覆盖最新制品；起点匿名解析仍失败，真实认证三方、全部书源兼容、指纹能力及生产切换尚未验收。

隔离本机测试可加 `--reader.server.bindAddress=127.0.0.1`，让 HTTP 服务仅监听回环地址；默认值仍为 `0.0.0.0`，不改变现有容器反向代理的连通性。仅改监听地址不会隔离 `storage/data`，仍须另设全新 `reader.app.workDir`。

## 新许可证中心

`license-server/` 是为 `https://license.medwarp.cn` 新建的独立服务。它不保存任何硬编码私钥、管理员令牌或 SMTP 凭据；SQLite、签名密钥和审计记录均放在部署时指定的持久化数据目录。

2026-09-19 已部署到 `cdn.medwarp.cn`：容器 `reader-license` 仅监听 `127.0.0.1:18087`，由现有宝塔 Nginx 的 `license.medwarp.cn` TLS 虚拟主机分流。生产密钥保存在 `/opt/reader-license/data`，恢复版客户端已经固定对应公钥。证书签发、自动续期 dry-run、容器重启持久性和公网端到端激活/吊销均已实测通过。

旧中心没有发现独立源码，原 JAR 也只包含公钥。不过，JAR 字节码和 `origin/feat/migrate-pro-features` 中的近似源码足以验证激活、在线校验、ReturnData 以及 RSA 分段格式。旧私钥无法从公钥恢复，因此：

- 恢复版继续尝试旧公钥，以便读取已有 `storage/data/license.key`；
- 新中心生成新的迁移密钥，恢复版通过 `reader.app.licensePublicKey` 固定其公钥；
- 未修改的原始 `reader-pro-3.2.14.jar` 不能接受新中心签发的许可证。

可重复构建：

```powershell
.\scripts\build.ps1

$env:JAVA_HOME = (Resolve-Path '.tools\jdk-11.0.8').Path
Push-Location license-server
.\gradlew.bat clean test fatJar --no-daemon
Pop-Location
```

部署和 API 说明见 `license-server/README.md`，恢复证据与验证边界见 `reports/LICENSE-CENTER-RECOVERY.md`。
