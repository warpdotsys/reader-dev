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

签名私钥和 SQLite 数据库均存于 `LICENSE_DATA_DIR`，必须纳入服务器加密备份；换私钥会使已签发 token 无法验签。管理 token 不写入数据库或日志。IP 只作为审计字段保存；当前服务刻意只信任 TCP 对端地址，不接受客户端可伪造的 `X-Forwarded-For`。因此当反代或 Reader 中转连接到本服务时，试用接口的每 IP 每小时 10 次额度是该 TCP 对端共享的总额度，不是每个最终用户各有 10 次。若日后需要按最终用户限流，必须另行实现仅接受受限反代来源的可信代理边界后再启用转发 IP。

## 已验证的旧路由外形

`POST /reader3/activateLicense` 与 `GET|POST /reader3/isLicenseValid` 已提供 `ReturnData` 外形。若设置 `LICENSE_LEGACY_RSA_ENABLED=true`，服务会加载 `LICENSE_LEGACY_RSA_PRIVATE_KEY_FILE` 指向的 PEM/DER PKCS#8 RSA 私钥，并从 `LICENSE_LEGACY_RSA_PUBLIC_KEY_FILE` 读取对应 X.509 公钥；未指定路径时仅在持久 data 卷首次生成 `legacy-rsa-private.pkcs8` 和 `legacy-rsa-public.spki`。其 `data.result` 使用已验证的 RSA-2048、PKCS#1 v1.5、私钥分段加密/Base64（包含 `=` padding、无换行）编码。

管理员可以调用 `POST /v1/legacy/licenses` 签发未激活的旧格式 License JSON 密文。该路由不伪造原上游密钥：它只适用于已经改为信任本服务新公钥的恢复版客户端。可从 `GET /.well-known/reader-license/legacy-public-key` 获取该公钥。

## 试用邮件（`/reader3` 兼容层）

`POST /reader3/sendCodeToEmail` 接受 `{ "email": "..." }`；成功的 ReturnData 仍为 `isSuccess=true`、`data=""`。`POST /reader3/supplyLicense` 接受 `{ "email": "...", "code": "8 位数字" }`；成功时返回可由恢复版 legacy 公钥验证的 `data.key`。它签发的试用 License 与原 JAR 行为一致：`type=trial`、`host=*`、15 个用户位、1 个实例、simple-web 到期时间为签发后 7 天。

验证码不是日志、HTTP 响应或数据库明文的一部分。服务以 HMAC 保存其摘要，15 分钟有效、每邮箱每小时最多 3 次、每 IP 每小时最多 10 次、全服务每小时最多 200 次、最多 5 次校验尝试；同时最多保留 1,000 个有效验证码与 100,000 个已领取邮箱。成功、过期或达到尝试次数即失效。长期领取表使用独立域分隔的 HMAC(email) 主键，而非额外保存明文邮箱；legacy License 的 `code=email` 为 JAR 兼容字段，仍保存在许可证记录中。领取状态保存在 SQLite，重启不会绕过一次性语义。

默认**禁用**邮件服务：只要没有 `LICENSE_SMTP_HOST`，两个路由都会返回 `isSuccess=false`、`errorMsg=试用邮件服务未配置`，不会伪造成功。生产环境须设置：

```text
LICENSE_SMTP_HOST=smtp.example.com
LICENSE_SMTP_PORT=587
LICENSE_SMTP_MODE=STARTTLS            # STARTTLS 或 SMTPS
LICENSE_SMTP_FROM=no-reply@example.com
LICENSE_SMTP_USERNAME=...
LICENSE_SMTP_PASSWORD_FILE=/run/secrets/reader-smtp-password
```

也可改为 `LICENSE_SMTP_PASSWORD`，但两者不能同时设置；推荐由容器 secret 挂载密码文件。密码文件只会移除**一个**末尾 LF 或 CRLF（便于常见 secret 文件），其他字符——包括开头或结尾空格——都作为密码的一部分保留。构建锁定 Jakarta Mail API 2.1.5、Angus Mail 2.0.5、Angus Activation 2.0.3。非本机模式必须同时配置用户名和密码，并且 Angus Mail 会要求 TLS、证书和主机名校验。仅隔离测试可设置 `LICENSE_SMTP_MODE=PLAINTEXT` 与 `LICENSE_SMTP_ALLOW_PLAINTEXT_LOOPBACK=true`，且 SMTP 主机必须是字面 `127.0.0.1` 或 `::1`，不能携带 SMTP 凭据；不接受 `localhost` 或任何可重解析的名称，以防 DNS 重绑定。这不是生产配置。服务不提供“跳过证书校验”开关。公网投递、供应商限额、SPF/DKIM/DMARC 和真实邮箱送达属于外部环境验收，不由本机 fake SMTP 测试宣称完成。

这不会使原始 JAR 自动兼容：原 JAR 内嵌的旧公钥与已丢失的旧私钥不匹配。恢复版客户端必须通过 `reader.app.licensePublicKey` 固定本服务公开的 legacy 公钥；原始 JAR 继续只作为行为对照。
