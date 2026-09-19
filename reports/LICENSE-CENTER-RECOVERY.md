# Reader 许可证中心恢复与重建报告

日期：2026-09-19（Asia/Shanghai）

## 结论

已经在 `license-server/` 从零建立独立许可证中心，把恢复版 Reader 的许可证客户端改到 `https://license.medwarp.cn`，并实际部署到 `root@cdn.medwarp.cn`。这不是空壳：生产链路已完成“管理签发未激活许可证 → Reader 经公网 HTTPS 调中心激活 → RSA 验签 → 写入 `storage/data/license.key` → 读取已安装许可证 → 中心在线校验 → 管理端吊销 → 签名状态返回已吊销”的端到端测试。

旧中心的独立服务端仓库没有找到；但协议并非猜测。原始 JAR 中同时保留了中心控制器字节码、客户端调用代码和原版前端，Git 历史 `origin/feat/migrate-pro-features` 又有逐步对齐 JAR 的可读 Kotlin 近似源码。新服务使用这些材料核对旧协议外形，但代码、数据层、管理认证和部署结构均重新实现。

## 已从 JAR 验证

- 后端客户端只联机调用：
  - `POST https://r.htmake.com/reader3/activateLicense`，请求 `{"content":"..."}`；
  - `GET https://r.htmake.com/reader3/isLicenseValid?id=...`。
- 原版前端另从浏览器跨域调用：
  - `POST /reader3/sendCodeToEmail`；
  - `POST /reader3/supplyLicense`。
- 成功响应为 `{"isSuccess":true,"errorMsg":"","data":...}`；激活和在线校验的 `data.result` 是签名密文。
- License 字段为 `host`、`userMaxLimit`、`expiredAt`、`openApi`、`simpleWebExpiredAt`、`instances`、`type`、`id`、`code`、`verified`、`verifyTime`。
- 算法是 RSA-2048、PKCS#1 v1.5，中心以私钥分段加密 UTF-8 JSON，客户端以公钥分段解密；明文块 245 字节、密文块 256 字节，最终使用带 `=` padding、无换行的标准 Base64。
- 原 JAR 内只有 X.509/SPKI 公钥，DER 294 字节，SHA-256 为 `2FF19229C1B3A2CF929E33A360F0C81C3E50FF6C745F363907ECFD39EADDF3AB`。
- 原 JAR、JAR 资源以及已检查的本地项目和下载目录均没有对应的旧私钥。原设计从中心运行目录的 `storage/data/privateKey.key` 读取 PKCS#8 私钥。

## 已从近似源码推断并交叉核对

- `origin/feat/migrate-pro-features` 当前为 `b05d7e54b1516ba70fd3d505299e813e09d1e067`；许可证文件历史中的 `61deb55`、`0cd34ad`、`27f763f` 等提交明确记录 JAR 对齐修复。
- `LicenseController.kt` 的激活次数、`type + code` 计数、10 分钟不同 IP 重复使用提示、定时校验和存储字段，与 JAR 的类、常量、路由和反编译控制流一致。
- 近似源码中曾存在公开的生成参数和内置邮件凭据。新服务没有复制这些不安全设计：管理签发、查询、吊销、审计全部要求 Bearer 管理令牌；SMTP 凭据没有进入仓库。

## 新中心实现

目录：`license-server/`

- Java 11、Gradle Wrapper、可独立运行的 fat JAR；
- SQLite 持久化许可证、实例激活、旧格式激活和审计；
- 数据目录持久保存两套密钥：现代 v1 的 `SHA256withRSA` 密钥，以及 Reader 3.2.14 迁移协议需要的 RSA 分段密钥；
- `GET /healthz`；
- `GET /.well-known/reader-license/v1/public-key`；
- `GET /.well-known/reader-license/legacy-public-key`；
- Bearer 管理 API：现代签发/查询/吊销/审计，以及 `POST /v1/legacy/licenses` 和旧格式吊销；
- Reader 兼容 API：`activateLicense`、GET/POST `isLicenseValid`；
- 原协议的 10 分钟跨 IP `repeat` 数据；
- 精确 Origin 白名单的 CORS/OPTIONS，支持原版前端的 `withCredentials` 预检；
- Dockerfile、Compose 和 Nginx 反代示例；容器以非 root 用户运行，数据卷由该用户持有；
- 私钥、数据库、`.env` 和构建目录均被忽略，不会误提交。

## 恢复版客户端改造

- 新增可配置项：
  - `reader.app.licenseServerUrl`，默认 `https://license.medwarp.cn`；
  - `reader.app.licensePublicKey`，部署时固定新中心的 legacy X.509 公钥；
  - `reader.app.licenseCheckEnabled`。
- 客户端保留原 JAR 公钥作为第二验证项，以读取可能仍存在的旧 `license.key`；新中心私钥永远不进入 Reader。
- 恢复 `getLicense`、`importLicense`、`isHostValid`、`decryptLicense`、试用请求代理和原 cron 在线校验。
- 许可证 HTTP 客户端启用系统 CA 和主机名校验，不复用项目中为书源兼容而设置 `trustAll=true` 的通用 WebClient。
- 原版压缩前端资源保持来源不变，只在 `processResources` 生成阶段把旧中心字面量替换为 `https://license.medwarp.cn`。最终 JAR 中旧 URL 计数为 0，新 URL 计数为 1。
- `importLicense` 等敏感路由的请求正文已从 INFO 日志中脱敏；实测日志只出现 `Request body: <redacted>`。

## 已成功重建与验证

### 恢复版 Reader

- `scripts/build.ps1` 已改为执行 `clean test bootJar`；
- 4 个测试、0 failure、0 error；其中新增许可证 host/过期行为测试 2 个；
- 首页、`getLicense`、`isHostValid` 本地启动探针均返回 HTTP 200；
- 产物：`build/libs/reader-4.0.7.jar`；
- 大小：72,896,918 bytes；
- SHA-256：`028A2D31D93AA520F3CC20076D86F8316561AE41F96A7ACA288CD8AE27DCF761`；
- 最终 JAR 内已固定生产中心 URL 和生产 legacy 公钥；公钥 SPKI SHA-256 为 `2F9F4D31C58F879CF1EB54EC9ECAF25632096D9241DF8E699209AB5DD7365B93`。

### 新许可证中心

- `gradlew.bat test fatJar --no-daemon` 成功，2 个测试、0 failure；
- 集成测试覆盖 CORS 预检、签发、激活、重复激活失败、空密钥失败、未知 ID、在线校验、吊销后失效及试用接口明确失败；
- 新增 DER 密钥重启回归测试，覆盖“首次生成后第二次启动仍可加载且密钥字节不变”；
- fat JAR 实际启动，`/healthz` 返回 `status=ok`；
- 产物：`license-server/build/libs/reader-license-server-0.1.0-all.jar`；
- 大小：16,470,008 bytes；
- SHA-256：`7D348ECBB121B50FF76F96A8F84B1F4A01F1D74F3923026A55D533B138EB4D53`；
- 归档任务已关闭文件时间戳并固定条目顺序；连续两次 `clean test fatJar` 得到相同 SHA-256，已验证为位级可重复构建。

### 两端端到端

- 管理 API 签发 `verified=false`、`type=pro`、`userMaxLimit=25`、`openApi=true` 的测试许可证；
- Reader 的 `/reader3/importLicense` 返回 `isSuccess=true`；
- Reader 落盘的 `storage/data/license.key` 为 344 bytes；
- `/reader3/getLicense` 读回 `verified=true`、`type=pro`、`userMaxLimit=25`、`openApi=true` 和中心生成的 UUID；
- 中心 `/reader3/isLicenseValid` 返回 `isSuccess=true` 且含签名 `data.result`。

## 已部署到生产服务器

- DNS：`license.medwarp.cn` CNAME 到 `cdn.medwarp.cn`，最终 A 记录为 `50.114.172.103`；
- 应用目录：`/opt/reader-license`；持久数据：`/opt/reader-license/data`；
- 容器：`reader-license`，镜像 `medwarp/reader-license:0.1.0`，`restart=unless-stopped`；
- 容器以 UID `10001` 运行，启用 `no-new-privileges`、丢弃全部 Linux capabilities，并限制 json-file 日志为 `10m × 3`；
- 后端只绑定 `127.0.0.1:18087`，未新增公网端口；
- 宝塔 Nginx 的公网 443 SNI 流量仍进入 `127.0.0.1:8443`，新增 `license.medwarp.cn` TLS 虚拟主机后反代到 `18087`，没有修改现有 `reader` 容器；
- Let’s Encrypt 证书 CN 为 `license.medwarp.cn`，当前有效期至 2026-12-18；Certbot 自动续期已存在，`renew --dry-run --run-deploy-hooks` 成功，部署钩子会检查并重载实际的宝塔 Nginx；
- 管理令牌仅在服务器 `/opt/reader-license/.env` 中生成和保存，权限 600，本报告和命令输出均未披露；
- 生产 legacy 公钥与恢复版 JAR 固定公钥的 SPKI SHA-256 均为 `2f9f4d31c58f879cf1eb54ec9ecaf25632096d9241df8e699209ab5dd7365b93`；
- 首次受控重建暴露了 DER 密钥重载会被误按 ASCII 读取的问题；密钥文件没有损坏，加载器已修复并加入重启回归测试后重新部署；
- 容器重建后又手工重启一次，两次均健康，公钥哈希保持不变；
- 服务器上原有 `reader` 容器保持 `healthy`，端口仍为 `4396 -> 8080`。

生产运维命令：

```bash
cd /opt/reader-license
docker compose ps
docker compose logs --tail=100 license
curl -fsS http://127.0.0.1:18087/healthz
/www/server/nginx/sbin/nginx -t -c /www/server/nginx/conf/nginx.conf
```

服务更新时，应先备份 `/opt/reader-license/data`（其中包含 SQLite 数据库和不可替换的签名私钥），再替换 fat JAR 并执行 `docker compose build && docker compose up -d`。不能重新生成或覆盖现有密钥，否则已签发许可证会全部失效。

## 兼容边界与尚未验证

- **原始 JAR 不能使用新中心的新密钥。** 缺少与 JAR 内置公钥配对的旧私钥时，这是 RSA 的密码学边界，不是可通过改域名解决的问题。未修改的 `reader-pro-3.2.14.jar` 继续只作为行为对照。
- 已有旧 `license.key` 仍可由恢复版的旧公钥验证；但旧中心在线校验和新中心数据库之间没有原激活记录，不能假装迁移完成。
- 邮箱验证码和 7 天试用签发尚未接入新的 SMTP 服务。两个旧路由现在返回 HTTP 200、`isSuccess=false`、`data=null` 和明确错误，不会假成功。
- 浏览器直连试用路由的 CORS 白名单当前为空；邮箱试用功能本身也尚未实现。未来接入 SMTP 时必须同时把实际 Reader 前端 Origin 加入 `LICENSE_CORS_ALLOWED_ORIGINS`，不能使用 `*` 配合凭据请求。
- 服务器现有 Nginx 配置在本次部署前后都报告一条 `cdn.medwarp.cn` 的 80 端口重复 `server_name` 警告；语法测试成功，且该警告不属于新增的 `license.medwarp.cn` 配置。本次没有擅自清理既有站点配置。

## 可重复构建

```powershell
cd C:\Users\chong\Documents\Codex\2026-09-16\g-i-t\work\reader-pro-restored
.\scripts\build.ps1

$env:JAVA_HOME = (Resolve-Path '.tools\jdk-11.0.8').Path
Push-Location license-server
.\gradlew.bat clean test fatJar --no-daemon
Pop-Location
```

生产公钥已经固定到恢复版 Reader；私钥文件、管理员令牌和未来 SMTP 凭据必须只留在服务器密钥/环境配置中。
