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

## 2026-09-23 简洁版页面补测与修复

在上一构建的隔离实例上，用可见浏览器打开 `/reader/simple-web/`，实际捕获到 `/reader3/getBookshelf`、`/reader3/getUserInfo` 两个根路径请求，均为 404。这是 `simple-web` 原版脚本把默认 API 固定为 `/reader3` 所致；主界面的路径处理不能覆盖这个独立入口。

本次在可读源码中按当前 `/simple-web/` 前缀计算默认 API，并为 JAR 内原始压缩脚本增加一个小型兼容脚本，置于通用脚本之后、页面脚本之前。查询参数、Cookie 或本地存储中显式设定的 API 地址不被覆盖；四个 HTML 入口由 `scripts/sync-simple-web-context-path.ps1` 同步更新，规则有独立的 Node 断言。未重编译或手工改写原始压缩脚本。

当前恢复构建 SHA-256：`D2360DD27DF837E2723A78C397336DD24FBC0B8602ABE909D70DB37254DC2D36`。`pwsh -NoProfile -File .\scripts\build.ps1` 与 `node scripts/test-simple-web-context-path.js` 均通过。用新的隔离子路径实例逐页复测：

| 页面 | 实际发出的接口 | HTTP |
| --- | --- | --- |
| 书架 | `/reader/reader3/getBookshelf`、`getUserInfo` | 200、200 |
| 搜索 | `/reader/reader3/getUserInfo`、`getBookSources` | 200、200 |
| RSS | `/reader/reader3/getUserInfo`、`getRssSources` | 200、200 |
| 阅读（无书籍参数） | `/reader/reader3/getBookshelf`、`getUserInfo` | 200、200 |

默认根路径的 `/simple-web/` 亦在独立实例复测，仍请求 `/reader3/getBookshelf` 与 `/reader3/getUserInfo`，均为 200。阅读页因未提供书籍参数，只验证空态及接口路径，**未验证真实章节阅读**。上述为本机浏览器/网络记录，不代表反向代理、生产域名或用户数据已经通过验收，#49 继续开放。
