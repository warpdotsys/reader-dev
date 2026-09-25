# 内置浏览器真实书源首轮探测

日期：2026-09-25。此记录是**未通过的探测**，不是原 JAR／远程 WebView 差分通过的证明。

从已部署 Reader 的 `transwarp/bookSource.json` 只读选取序号 123（“过期杂志（优+）”）。该书源启用、搜索 URL 要求 WebView，不带登录或自定义请求头；在执行前检查了规则，不含 `@js:`、`<js>` 或 `javascript:`。只把这一条规则送入本机临时 Reader 工作目录和随机测试账号；未修改线上数据，也未保存书源规则、网页正文或账号凭据。诊断使用本机已安装的 Chrome；正式镜像仍须使用镜像内的 Chromium。

运行方式（此模式**仅测试恢复版**，不会暗示原 JAR 一致）：

```powershell
python -B scripts\compare-live-source-reading.py `
  --source-namespace transwarp --source-index 123 --query 读者 `
  --local-browser-only `
  --browser-executable 'C:\Program Files\Google\Chrome\Application\chrome.exe' `
  --report reports\local-webview-live-source-123-reader.json --quiet
```

“三国演义”和“读者”两次搜索均返回 HTTP 200、`isSuccess=true`、零条结果；因此书籍详情、目录、正文尚未测到。独立 GET 站点首页得到 HTTP 200；对“读者”的搜索页 GET 也得到 HTTP 200，但当前 HTML 中找不到该书源 `ruleSearch` 依赖的 `.ar_title` 类。现有证据倾向于这条书源的规则已随站点改版失效，**不能**据此认定内置浏览器正确或错误。两次完整的非敏感摘要分别见 `local-webview-live-source-123.json` 和 `local-webview-live-source-123-reader.json`。

下一步需要使用仍有有效结果的 WebView 书源，在隔离容器中比较原 JAR／远程实现／内置浏览器的搜索、请求方法和正文；含书源脚本的候选不可直接在本机用户权限下运行。还须覆盖 Cookie、超时、资源上限和隔离。生产入口未切换。
