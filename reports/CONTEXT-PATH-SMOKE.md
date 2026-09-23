# 子目录入口本机回归

验证日期：2026-09-23。测试对象是当前恢复工程构建的 JAR；本轮**没有**把原始 `reader-pro-3.2.14.jar` 或生产域名当作已验证对象。

构建：`pwsh -NoProfile -File .\scripts\build.ps1`，产物 SHA-256 为 `EC442A6BC33DD9E3022C30339B1B9E9D513455C8467782BAAE02DC4A6F60F90B`。回归：`pwsh -NoProfile -File .\scripts\verify-context-path.ps1`。脚本在独立临时工作目录启动根路径和 `/reader` 两个实例，不挂载用户数据；成功后停止实例并清理测试目录。

## 已从恢复版复现和修复

修复前，`GET /reader` 返回 200 首页，但页面的相对 CSS/JS 地址会被浏览器解析到站点根目录，根目录资源返回 404。修复后，`GET /reader` 返回 308，`Location: /reader/`；查询串如 `?x=1` 保留。`GET /reader/` 返回 200，不再重定向循环。

根路径模式下，首页、CSS、JS 和 `/reader3/getSystemInfo` 均为 200。子路径模式下，首页、CSS、JS、service worker、manifest、`/reader/assets/reader.css` 与 `/reader/reader3/getSystemInfo` 均为 200；站点根目录的 CSS 为 404，证明检查没有意外依赖根路径资源。

## 尚未验证

- 本轮是 HTTP 黑盒回归，未做完整浏览器登录、书架、阅读和 PWA 离线交互。
- 未验证反向代理对前缀、重定向和 `Location` 的处理，也未部署到 `read.medwarp.cn`。
- 书籍封面、下载响应及所有服务端生成的绝对资源 URL 仍需逐项检查。因此 [#49](https://github.com/warpdotsys/reader-dev/issues/49) 保持开放。
