# WebView 黑盒差分与浏览器候选核查

验证日期：2026-09-23（Asia/Shanghai）。运行命令：

```powershell
python -B .\scripts\compare-webview-cookie.py
```

脚本先后启动原始 JAR 与恢复构建，均使用独立临时工作目录和随机合成账号。唯一书源、远程 `/render.html` 服务与三次搜索都在本机 `127.0.0.1`；没有访问第三方站点、生产书源或用户数据，也没有下载浏览器。生成的机器可读结果见 `reports/webview-cookie-diff-latest.json`。

## 已从原始 JAR 验证

原 JAR SHA-256：`B26FB4769D689D98FF26408CE79A275D719F360906C84ACF52FF404E98030C8C`。现代格式书源中保留 `ruleToc` 后，`searchUrl` 的 WebView 选项由原 JAR 正常解析，三次 `/reader3/searchBook` 均为 HTTP 200、`isSuccess=true`、`errorMsg=""`，各返回 1 条固定书目。模拟远程渲染器接收到的三次 `Cookie` 请求头均为空。首次响应提供了 `session=alpha==; Path=/; HttpOnly; SameSite=Lax`，第二次响应提供删除 Cookie 的 `Max-Age=0`；这与先前字节码核查发现的原 JAR `_cookieJar` 写入无效一致。

## 已成功重建

恢复 JAR SHA-256：`6F6CAB12229CE6729B380F261F23183423337D1F4ABC9C30EB6FEC4ADA70A40E`。三次搜索的 HTTP 状态、`isSuccess`、`errorMsg` 与结果数量与原 JAR 一致。模拟远程渲染器收到的 Cookie 依次为：空、`session=alpha==`、空。第二次未携带 `Path`、`HttpOnly` 或 `SameSite` 属性；第三次在删除响应后不再携带 Cookie。这是有意修复的差异，不应记作逐字节兼容。

第一次夹具尝试只提供 `ruleSearch`，原 JAR 将其识别为旧格式并丢掉 `searchUrl`；加入最小 `ruleToc` 后已复跑成功。此处确认的是书源格式判别逻辑，不把错误夹具的空结果归咎于 WebView 功能。

## 候选现状与决定

- [Playwright Java 官方文档](https://playwright.dev/java/docs/browsers)说明每个 Playwright 版本需要匹配浏览器二进制，单个浏览器下载可能达数百 MB。因此本轮不在用户电脑上临时拉取浏览器；普通 Chromium 仅作为功能和资源基线，不冒称具备内核级指纹能力。
- [Playwright Java Docker 文档](https://playwright.dev/java/docs/docker)明确其镜像偏测试/开发用途；访问不可信站点推荐非 root 用户及 seccomp，Firefox/WebKit 不支持 Alpine/musl。浏览器版镜像须另设 glibc 基线和沙箱验收，不套用现有 Alpine 轻量镜像。
- [Camoufox 官方跨语言服务文档](https://camoufox.com/python/remote-server/)将远程 WebSocket 服务标为实验性，并指出单个服务实例不会按会话轮换指纹。[安装文档](https://camoufox.com/python/installation/)要求另行获取浏览器二进制；其示例环境的浏览器目录为 1.2 GB，但不应把示例数值当成未来版本的固定体积。因协议稳定性、会话隔离和成本未实测，本轮没有选定 Camoufox，也没有下载它。

## 尚未验证

- 已部署的真实 WebView 书源、登录/代理/JS/重定向/编码以及多用户并发。此次远端 SSH 连接超时，未从服务器读取书源，也没有修改线上服务。
- 浏览器进程启动、指纹配置、容器沙箱、CPU/内存、崩溃恢复与多架构镜像。本脚本模拟的是现有 `/render.html` 协议，不包含真正浏览器，不能作为“内置浏览器已完成”的证据。
- 构建产物的字节级可重复性。报告中的恢复 JAR 散列仅定位本机受测产物，不等于发布制品散列。
