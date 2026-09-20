# 请求日志安全处置记录

处置日期：2026-09-20

生产域名：`https://read.medwarp.cn`

修复版本：`v4.0.7-restored.7`
GitHub Actions：[run 35504681394](https://github.com/warpdotsys/reader-dev/actions/runs/35504681394)

## 已确认的问题

旧版 `RestVerticle` 会记录完整请求 URI，并在部分请求上记录较短的正文。因此，查询参数中的认证信息以及登录请求正文可能进入应用日志。

生产处置前仅做计数检查，不输出值：

- 应用日志中 `accessToken=` 出现 6 次。
- 应用日志中旧格式 `Request body:` 记录出现 4 条。
- 实际 Nginx 日志目录 `/www/wwwlogs` 中未发现 `accessToken=`。
- 两份受影响应用日志总大小约 142 MB。

## 已成功重建并发布

- 请求日志只记录 HTTP 方法、去除查询串的规范化路径、状态码和耗时。
- 不再记录请求正文。
- 过滤路径中的回车、换行和制表符，并将路径长度限制为 2,048 字符。
- 自动化测试覆盖查询串去除和日志控制字符处理。
- 发布由 GitHub 托管 Runner 完成：构建使用 `ubuntu-24.04`，生产部署使用 `macos-15`，发布使用 `ubuntu-24.04`。
- 生产服务器没有安装 GitHub Actions 自托管 Runner。

## 生产验证

使用不含真实凭据的唯一探针分别访问查询参数接口和登录接口。发布后结果：

- 应用日志中的查询探针：0。
- 应用日志中的正文探针：0。
- Nginx 日志中的查询探针：0。
- Nginx 日志中的正文探针：0。
- 首页和 `/reader3/getSystemInfo` 均返回 HTTP 200，响应通过严格 UTF-8 解码。
- `/reader3/getSystemInfo` 返回可解析 JSON，`isSuccess=true`。
- 生产容器健康状态为 `healthy`。

## 历史日志脱敏

脱敏时短暂停止 `reader-pro-restored` 容器，逐行流式处理明确指定的两份 `reader-*.log`，在同一目录原子替换后再启动容器。未保留未脱敏副本。

- `reader-2026-09-19.log`：274 处替换。
- `reader-2026-09-20.log`：106 处替换。
- 合计：380 处替换。该数字包含被整体移除的旧请求查询串，不等同于泄露凭据数量。
- 处置后 `accessToken=` 残留为 0，未脱敏 `Request body:` 残留为 0。
- 日志目录权限为 `750`，日志文件权限为 `640`，属主保持 `10001:10001`。
- 重启后容器恢复 `healthy`。

可审计的脱敏工具保存在 `scripts/redact-reader-logs.py`。工具只接受 `/opt/reader-pro-restored/logs/reader-*.log` 下的普通文件，拒绝符号链接和目录外路径；替换过程保持原权限与属主，不创建含原始内容的备份。

## SSH 收尾

- 标准 22 端口继续使用公钥登录。
- 禁用密码及键盘交互认证，root 仅允许公钥认证。
- 测试 GitHub 托管 Runner 连通性时临时使用的 22222 SSH 监听、systemd 单元以及 UFW IPv4/IPv6 放行规则均已删除。
- 删除后再次验证 22 端口公钥登录成功、22222 无监听、SSH 配置语法有效。

## 尚未验证与后续动作

- 服务器或基础设施提供商在本次处置前生成的快照、离线备份不在此次在线日志清理范围内。
- 本次没有自动修改用户密码或令牌，以免破坏现有客户端会话和数据。若密码曾用于其他系统，应主动更换；若需要强制失效既有令牌，应在确认业务影响后单独执行。
- WebDAV、SSE、下载、第三方书源及全部写接口仍需按发布说明继续回归。

## 复查命令

只统计标记，禁止输出匹配行或真实值：

```bash
grep -I -F -h -o -- 'accessToken=' /opt/reader-pro-restored/logs/reader-*.log | wc -l
grep -I -F -h -- 'Request body:' /opt/reader-pro-restored/logs/reader-*.log \
  | grep -Fv -- '<redacted historical entry>' | wc -l
docker inspect -f '{{.State.Health.Status}}' reader-pro-restored
```
