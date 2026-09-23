# 原始 JAR 与恢复版：认证和用户存储差分

验证日期：2026-09-23（Asia/Shanghai）

## 方法

运行 `scripts/compare-auth-lifecycle.ps1`，分别启动原始 JAR 和当前源码干净构建的恢复 JAR。两端使用独立的临时工作目录、随机端口和临时生成的测试账号；不读取生产账号密码，也不写入生产数据。脚本结束时停止两个测试进程，测试目录保留在被 Git 忽略的 `.tools/` 下，方便复核。

```powershell
pwsh -NoProfile -File .\scripts\build.ps1
pwsh -NoProfile -File .\scripts\compare-auth-lifecycle.ps1
```

本次原始 JAR SHA-256：`B26FB4769D689D98FF26408CE79A275D719F360906C84ACF52FF404E98030C8C`。

本次恢复构建 SHA-256：`543F9480C9B7E3DD60638B842D22EF88C7D97A4A70D4BF9B2D268289CD0F0248`。

## 已从 JAR 验证并与恢复版比对

下列 11 个探针的 HTTP 状态、`Content-Type`、`ReturnData` 成败/错误文案/数据类型、相关字段集合及关键值，在两端语义一致：

| 探针 | 结果 |
| --- | --- |
| 不存在用户登录 | 两端 HTTP 200，`isSuccess=false`，`用户不存在` |
| 新用户注册 | 两端成功，返回相同用户字段集合和 `用户名:令牌` 格式 |
| 错误密码登录 | 两端 HTTP 200，`isSuccess=false`，`密码错误` |
| 正确密码登录 | 两端成功，返回相同用户字段集合 |
| Cookie 会话读用户信息 | 两端成功，包含用户信息，安全模式为 true |
| accessToken 读用户信息 | 两端成功，字段结构一致 |
| accessToken 读空书架 | 两端成功，空数组 |
| Cookie 会话写用户配置 | 两端成功 |
| accessToken 读用户配置 | 两端成功，包含相同字段、测试值及数值型更新时间 |
| 带 accessToken 注销 | 两端返回相同 `ReturnData` |
| 注销后旧令牌读书架 | 两端失败，`请登录后使用` |

两端写出的用户文件路径也一致：`storage/data/users.json` 及 `storage/data/<测试用户>/userConfig.json`。用户配置文件均包含 `@updateTime`、`options`、`probe`，测试值一致。动态令牌与毫秒时间戳只比较格式/类型，不要求字节相同；脚本的报告不包含密码或令牌值。

机器可读的逐项结果见 `reports/auth-lifecycle-diff-latest.json`。脚本遇到响应语义或存储结构差异会以非零状态退出。

## 源码结构核对

对 `UserController.class` 的 `javap -p` 抽样显示，恢复版保留原 JAR 的公开方法签名，包括登录、注销、用户管理、配置、文件和备份接口。恢复版另有一个用于限制资源路径的私有辅助方法，不改变公开 ABI。

`YueduApi.class` 常量池中的 `/reader3/` 路由字符串分别为原 JAR 112 个、恢复版 108 个。差出的 4 条是旧中心的 `activateLicense`、`generateKeys`、`generateLicense`、`isLicenseValid`；目前签发和在线校验由独立许可证中心承担，不能把这些路由差异误记为普通阅读接口缺失。常量池计数只覆盖字面量，仍需逐路由做行为测试。

对其余关键控制器的 `javap -p` 抽样：

| 控制器 | 公开/受保护 ABI 对比 |
| --- | --- |
| `FileController`、`WebdavController` | 原 JAR 与恢复构建的签名集合相同 |
| `BookController` | 原 JAR 的业务方法均在恢复版；差异为编译器合成访问器和恢复版新增的缓存/并发属性访问器，需继续做行为差分 |
| `BookSourceController` | 原 JAR 的签名均在恢复版；恢复版另有一个 Kotlin 默认参数桥接方法 |
| `HttpTTSController` | 原 JAR 的签名均在恢复版；恢复版另有四个列表增删改相关方法 |
| `LicenseController` | 签名不同：旧中心的生成/激活/校验方法已迁至独立服务；试用路由的实现和返回型也不同，须继续单独验证 |

以上是公开 ABI 清单比较，不能证明同名方法体与原 JAR 相同，也不覆盖反射调用或私有逻辑。

## 尚未验证

- 本测试使用隔离空数据，只覆盖认证和一项用户配置写入；没有检验生产账号的全部数据内容。
- WebDAV、文件上传/下载、SSE、书源解析、许可证签发和 EPUB/TXT 导出仍需独立差分。
- 原 JAR 与恢复版的内部实现可能不同；上述结果只证明列出的外部行为一致。
