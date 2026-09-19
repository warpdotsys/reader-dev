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
