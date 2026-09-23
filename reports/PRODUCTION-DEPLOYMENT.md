# Reader Pro 生产部署记录

部署日期：2026-09-19  
目标主机：`root@cdn.medwarp.cn`  
公网域名：`https://read.medwarp.cn`

## 构建与版本

- Git 默认分支：`legacy`
- 恢复工程提交：`c71a5a5639273dc51d9aa5bd0304685f896c833a`
- Reader 产物：`reader-4.0.7.jar`
- SHA-256：`2DCC4A35DE955F7957DB30B1D6D4D7E61E00A4E2C7AAF02CD8EA9EBCD3BDD9DA`
- 构建命令：`powershell -ExecutionPolicy Bypass -File .\scripts\build.ps1`
- 构建验证：Gradle 构建成功，4 个测试全部通过
- 许可证服务构建验证：`clean test fatJar` 成功，2 个测试全部通过

## 部署拓扑

- 部署目录：`/opt/reader-pro-restored`
- Docker 镜像：`medwarp/reader-pro-restored:4.0.7`
- 容器：`reader-pro-restored`
- 应用监听：`127.0.0.1:18088 -> 8080/tcp`
- Nginx 虚拟主机：`/etc/nginx/conf.d/read.medwarp.cn.conf`
- TLS：Let's Encrypt，证书到期日 2026-12-18
- 自动续期：Certbot 模拟续期成功，续期部署钩子完成 Nginx 配置检查与重载

旧 Rust 容器 `reader` 仍在端口 4396 健康运行，原有 `/home/transwarp/reader/storage` 未修改、未覆盖。新域名只指向恢复的 Java/Kotlin 版本。

## 数据导入

源数据：`D:\Download\storage\data`。

- 文件数：55,692
- 文件内容总字节数：780,079,185
- 传输归档：未压缩 TAR，避免中文文件名在 ZIP 解包时发生编码损失
- TAR SHA-256：`BB0754AC8CD143919549826CFA162D4CA1AFD41DC480762A7FE0BF93E0006573`
- 服务端导入位置：`/opt/reader-pro-restored/storage/data`
- 服务端复核：文件数和总字节数均与源数据一致，中文路径抽样存在

首次 ZIP 尝试因服务端解包出现中文文件名不一致而被拒绝，没有作为生产数据使用。生产验证完成后，服务端上传归档和不完整解包目录已清理；源数据、正式导入目录和旧 Rust 数据均未改动。

## 验证结果

### 已成功重建并实测

- 容器健康检查通过，应用完成 Spring Boot 启动。
- 源站 HTTPS 首页返回 200，响应体 5,625 字节。
- 公网 `https://read.medwarp.cn/` 返回 200，TLS 校验通过。
- 公网 `/reader3/getSystemInfo` 返回 `isSuccess=true`、空 `errorMsg`、对象型 `data`。
- 使用导入数据中的有效 accessToken 验证（令牌未记录）：
  - `/reader3/getUserInfo`：成功
  - `/reader3/getBookshelf`：成功，169 项
  - `/reader3/getBookSources?simple=1`：成功，429 项
- 新容器与旧 Rust 容器同时为 healthy。

### 尚未验证

- 所有写操作、WebDAV、SSE 长连接、下载大文件及第三方书源的完整生产行为尚未逐项回归。
- 本次只验证了导入账户中的一个有效 accessToken；未记录或输出凭据。
- 现有 Nginx 配置中有与本项目无关的 `cdn.medwarp.cn` 重复 server_name 警告；不影响本域名配置测试、启动和证书续期，但应在服务器维护窗口单独清理。

## 回滚

1. 将 `/etc/nginx/conf.d/read.medwarp.cn.conf` 替换为同目录的 `read.medwarp.cn.conf.http-only-backup-20260919`，或把上游改回指定旧服务。
2. 执行 `/www/server/nginx/sbin/nginx -t -c /www/server/nginx/conf/nginx.conf`。
3. 配置检查通过后执行 `/www/server/nginx/sbin/nginx -s reload -c /www/server/nginx/conf/nginx.conf`。
4. 停止新容器不会影响仍在运行的旧 `reader` 容器及其原有数据。

## 2026-09-20 GitHub 托管 Runner 发布更新

- 当日生产标签：`v4.0.7-restored.7`。
- 当日制品 SHA-256：`9bdfe481c01ec9e0636ff7b34f691ed6f5773c32ad0e82633c2d744d6d238933`。
- GitHub Actions：[run 35504681394](https://github.com/warpdotsys/reader-dev/actions/runs/35504681394)。
- GitHub Release：[v4.0.7-restored.7](https://github.com/warpdotsys/reader-dev/releases/tag/v4.0.7-restored.7)。
- 构建、部署和发布均使用 GitHub 托管 Runner；生产服务器未安装自托管 Runner。
- 部署作业通过标准 22 端口公钥 SSH 连接，测试期临时 22222 监听及防火墙规则已删除。
- 当日生产容器 `reader-pro-restored` 健康，首页及 `/reader3/getSystemInfo` 公网复验通过严格 UTF-8 解码。
- 请求日志安全修复与历史日志处置证据见 `reports/SECURITY-LOGGING-REMEDIATION.md`。

## 2026-09-23 导入资源目录修复

浏览器实测发现用户封面 URL 返回 404。导入源 `D:\Download\storage\data` 中包含 `assets/` 子目录；服务器原样导入后，资源实际位于 `/opt/reader-pro-restored/storage/data/assets`，而 Java/Kotlin 服务固定从 `/opt/reader-pro-restored/storage/assets` 提供 `/assets/*`。这是部署目录布局问题，文件本身未丢失。

- 将嵌套目录中的 130 个文件（66,654,762 bytes）复制到服务读取目录；源文件保留，不覆盖其它已有资源。
- 原 `reader.css` 是 3,535 字节的用户自定义样式；线上 54 字节文件是应用自动生成的默认样式。替换前已把默认文件备份到 `/opt/reader-pro-restored/releases/data-assets-repair-20260923/reader.css.before-repair`。
- 修复后，源目录和正式资源目录的 130 个同名文件经 `rsync -rnc` 校验内容差异为 0；自定义样式 SHA-256 为 `D39266E9FF7186F22C20942B2333104E7B3C2F6E8E06D581F7D5D6CCF417DBF4`。
- 容器重启清除旧静态资源缓存后，公网封面 URL 返回 200、48,905 字节；`/assets/reader.css` 返回 200、3,535 字节。浏览器实际显示了此前缺失的封面，容器状态为 `healthy`。
- 另有一条 `/assets/covers/...` 封面 URL 在导入数据中无对应文件，仍返回 404；前端使用原版占位图。本次没有凭空生成或改写书籍记录。

未来迁移 `storage/data` 时，需要检查是否含有嵌套 `assets/`，并把它同步至正式 `storage/assets/`；若正式位置已有自定义文件，应先逐文件比对，避免覆盖用户更改。

## 2026-09-23 v4.0.7-restored.8 发布

- GitHub 托管 runner 的构建、生产部署、公网检查和 Release 发布全部成功：[发布流水线](https://github.com/warpdotsys/reader-dev/actions/runs/35833753491)、[发布说明与制品](https://github.com/warpdotsys/reader-dev/releases/tag/v4.0.7-restored.8)。
- 服务器 `DEPLOYED_RELEASE` 标记为 `v4.0.7-restored.8`；标记与当前 JAR 的 SHA-256 同为 `D2AA722284965C7B05544C98C49B509B2E52F37CC0487FB860FFC75D7B53D780`；容器状态 `running/healthy`。
- 公网匿名、只读的真实书源搜索与详情均为 HTTP 200；搜索 100 条，结果摘要与本地恢复版一致，搜索响应不再暴露 `_userNameSpace`。
- 生产登录态目录、正文、书架和缓存链路仍未验证。使用现有 accessToken 自动登录会更新 `users.json`，故未将其视为无副作用检查。完整差分与边界见 `reports/LIVE-SOURCE-READING-DIFF.md`。
