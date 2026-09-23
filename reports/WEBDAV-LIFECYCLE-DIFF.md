# 原始 JAR 与恢复版：WebDAV 生命周期差分

验证日期：2026-09-23（Asia/Shanghai）。

## 方法

使用 `scripts/compare-webdav-lifecycle.py`，分别启动原始 JAR 的只读备份和当前源码的干净重建产物。两端各用独立临时工作目录、随机端口及脚本生成的测试账户；启动参数仅为该测试账户开启 WebDAV。脚本终止 Java 进程后自动删除临时目录。不使用生产账号、密码、令牌或数据。

```powershell
pwsh -NoProfile -File .\scripts\build.ps1
python -B .\scripts\compare-webdav-lifecycle.py
```

原始 JAR SHA-256：`B26FB4769D689D98FF26408CE79A275D719F360906C84ACF52FF404E98030C8C`。本次干净重建 JAR SHA-256：`CD81FC945928E9E558BCEED6970AD152E500BF0CA92128ECE5C34E4FEC3B3B89`。

## 已从 JAR 验证并与恢复版比对

19/19 个探针在 HTTP 状态、相关响应头及规范化响应结构上相同：匿名 `OPTIONS`、匿名与错误密码 `PROPFIND`、授权根目录和子目录 `PROPFIND`、`MKCOL`、普通及中文文件名的 `PUT`/`GET`、`COPY`、`MOVE`、`LOCK`/`UNLOCK`、文件和目录 `DELETE`、删除后 `GET`。脚本不仅检查两端相等，还断言预期状态码、上传/下载字节 SHA-256、目录条目及清理后的空目录。动态的锁 UUID、端口和文件时间不要求字节相同，但校验其形态与关联关系。

两端 WebDAV 目录均位于 `storage/data/<测试用户>/webdav`，删除后没有残留测试文件。逐项规范化结果见 `reports/webdav-lifecycle-diff-latest.json`。原 JAR/恢复版均将匿名预检 `OPTIONS` 返回 200，而其他受保护操作在未认证或密码错误时返回 401。

公网 `https://read.medwarp.cn/reader3/webdav/` 的匿名、无正文检查也分别得到 `OPTIONS` 200、`PROPFIND` 401，确认当前 Nginx 路由允许这两种方法通过；未向生产服务发送 Basic 凭据或文件写入请求。

## 尚未验证

- 除匿名 `OPTIONS`/`PROPFIND` 外，这是隔离环境差分，不等于生产域名经过 Nginx 的完整 WebDAV 客户端实测，也不证明导入账户已启用 WebDAV。
- 尚未覆盖两个用户之间的命名空间、覆盖已有目标、缺失父目录、超大文件、并发上传、备份恢复，以及不同 WebDAV 客户端兼容性。
- 没有为追求原版一致性放宽路径校验；符号链接和路径穿越安全边界需单独审计。
