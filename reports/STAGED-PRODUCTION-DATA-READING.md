# 生产导入数据的隔离登录态阅读验证

验证日期：2026-09-23（Asia/Shanghai），线上版本 `v4.0.7-restored.8`。本测试没有向生产容器发送登录态请求，也没有修改正式 `storage/data`。

在生产主机上建立一个权限为 `700` 的临时目录，仅复制一个既有账户记录、一条无需额外登录的书源和该账户的书架文件；文件权限为 `600`。脚本先核对临时容器所用镜像 ID 与当前生产容器一致；临时容器只监听服务器本机 `127.0.0.1` 的随机端口，限制为 2 CPU / 1 GiB 内存。现有 accessToken 只在服务器内存与该临时实例中使用，未输出到命令结果、仓库或报告。自动登录更新的是隔离副本的 `users.json`。

复现命令（将 `YOUR_NAMESPACE` 替换为已有账户目录名；需要服务器公钥 SSH）：

```powershell
Get-Content -LiteralPath .\scripts\verify-staged-production-data.py -Raw -Encoding utf8 |
    ssh -T root@cdn.medwarp.cn "python3 - --namespace YOUR_NAMESPACE --source-index 98"
```

## 已成功重建并实测

- 已导入账户的 token 认证成功，隔离实例读取书架 169 项，与原书架文件条目数一致。
- 同一真实书源搜索返回 100 条，书籍详情取得目录地址，目录返回 1914 章。
- 第一章正文长度 2852 个 Unicode 字符，UTF-8 SHA-256 为 `27D041EDEDD93E919F4D3DE969A0E13061E40001C84DA51BDCE12068348B45E1`，与此前原始 JAR 和恢复版的本机差分结果一致；正文文本未保存或展示。
- 正式 `users.json`、用户书源文件和书架文件在测试前后的 SHA-256 均一致。脚本清理了临时容器及副本；独立复核未发现残留测试容器或 `auth-reading-*` 目录，正式容器仍为 `running/healthy`。
- 正式容器内 `/app/reader.jar` 与服务器部署目录的 JAR SHA-256 一致，均为 `D2AA722284965C7B05544C98C49B509B2E52F37CC0487FB860FFC75D7B53D780`。

## 尚未验证

这证明当前发布镜像能在同一服务器上读取生产导入账户、书源和书架的受限副本，并完成真实站点阅读；不是“生产容器本体的登录态 HTTP 请求已通过”。前端实际点击、其他账户、书架内具体书目的进度、WebDAV，以及 EPUB/PDF/CBZ 等仍待覆盖。隔离副本只含选定书源，不代表完整 429 条书源同时运行的状态。
