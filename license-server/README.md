# Reader 许可证服务（v1）

这是 `license.medwarp.cn` 的独立、可审计服务。它同时提供新的 v1 API，及已按 JAR 字节码核实的 `/reader3` 迁移兼容层：ReturnData、RSA-2048 PKCS#1 v1.5 分段私钥加密、激活和在线校验。它不是旧中心私钥的恢复品；原始 JAR 必须先改为信任本服务的新公钥。

## 构建与本地运行

```powershell
./gradlew.bat test fatJar
$env:LICENSE_ADMIN_TOKEN = 'use-a-long-random-secret'
$env:LICENSE_DATA_DIR = "$PWD/data"
java -jar build/libs/reader-license-server-0.1.0-all.jar
```

健康检查：`GET /healthz`。生产部署须由反向代理为 `https://license.medwarp.cn` 终止 TLS；Compose 只把容器端口发布到宿主机 `127.0.0.1:8080`。

浏览器前端需要跨域时，设置 `LICENSE_CORS_ALLOWED_ORIGINS=https://你的-reader-域名`（多个来源用逗号分隔）。只有精确匹配的 Origin 才会获得带凭据的 CORS 响应和 OPTIONS 预检许可；默认不放行任何来源。

## v1 API

管理接口均要求 `Authorization: Bearer $LICENSE_ADMIN_TOKEN`：

* `POST /v1/licenses`：创建并签发 RSA-2048 / SHA-256 签名 token；请求字段为 `subject`、`host`、`expiresAt`（毫秒，0 表示永久）、`userMaxLimit`、`instances`、`features`。
* `GET /v1/licenses/{id}`，`POST /v1/licenses/{id}/revoke`：查询和吊销。
* `GET /v1/audit`：最近 200 条审计记录。
* `POST /v1/legacy/licenses`：签发迁移用的未激活旧格式密文；`POST /v1/legacy/licenses/{type}/{code}/revoke`：吊销该迁移许可证。

客户端接口：`POST /v1/activations`、`POST /v1/validations`。请求必须包含 `token` 和 `instanceId`。Token 格式 `rlic1.<base64url JSON payload>.<base64url RSA/SHA-256 signature>`；公钥见 `GET /.well-known/reader-license/v1/public-key`。

签名私钥和 SQLite 数据库均存于 `LICENSE_DATA_DIR`，必须纳入服务器加密备份；换私钥会使已签发 token 无法验签。管理 token 不写入数据库或日志。IP 只作为审计字段保存，部署时应由受信任反代设置 `X-Forwarded-For`。

## 已验证的旧路由外形

`POST /reader3/activateLicense` 与 `GET|POST /reader3/isLicenseValid` 已提供 `ReturnData` 外形。若设置 `LICENSE_LEGACY_RSA_ENABLED=true`，服务会加载 `LICENSE_LEGACY_RSA_PRIVATE_KEY_FILE` 指向的 PEM/DER PKCS#8 RSA 私钥，并从 `LICENSE_LEGACY_RSA_PUBLIC_KEY_FILE` 读取对应 X.509 公钥；未指定路径时仅在持久 data 卷首次生成 `legacy-rsa-private.pkcs8` 和 `legacy-rsa-public.spki`。其 `data.result` 使用已验证的 RSA-2048、PKCS#1 v1.5、私钥分段加密/Base64（包含 `=` padding、无换行）编码。

管理员可以调用 `POST /v1/legacy/licenses` 签发未激活的旧格式 License JSON 密文。该路由不伪造原上游密钥：它只适用于已经改为信任本服务新公钥的恢复版客户端。可从 `GET /.well-known/reader-license/legacy-public-key` 获取该公钥。`POST /reader3/sendCodeToEmail` 与 `POST /reader3/supplyLicense` 会明确返回未配置试用邮件服务的 ReturnData 错误，不会假成功。

这不会使原始 JAR 自动兼容：原 JAR 内嵌的旧公钥与已丢失的旧私钥不匹配。恢复版客户端必须通过 `reader.app.licensePublicKey` 固定本服务公开的 legacy 公钥；原始 JAR 继续只作为行为对照。
