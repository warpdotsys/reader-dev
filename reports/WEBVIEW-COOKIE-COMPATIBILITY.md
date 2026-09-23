# WebView Cookie 兼容修复

记录日期：2026-09-23。当前改动只在源码与本机定向测试中，未发布、未部署。

## 已从原始 JAR 验证

对只读备份 `reference/original/reader-pro-3.2.14.original.jar` 的 `RemoteWebview`、`CookieStore`、`NetworkUtils` 使用 JDK 11 `javap -c -p` 核对：远程渲染收到 `Set-Cookie` 后，把 `域名_cookieJar` 传给 `CookieStore.replaceCookie`；后者最终调用 `NetworkUtils.getSubDomain` 作为磁盘缓存键。`getSubDomain` 先调用 `getBaseUrl`，而 `getBaseUrl` 对不以 `http` 开头的字符串返回 `null`。因此这条 `_cookieJar` 写入路径在原 JAR 中无法产生缓存键。这是原版已有的缺陷，不是恢复工程新引入的差异。

## 已从当前源码确认并成功重建

`AnalyzeUrl` 在后续请求前会先读取 `域名_cookieJar`，再拷贝到普通域名 Cookie；旧实现对这两种非 URL 键都读不到。`CookieStore` 现在保留完整 URL 的原有域名归一化行为，同时接受受限字符集的裸域名和 `_cookieJar` 键；磁盘缓存仍沿用 ACache 及原有用户命名空间路径，不迁移既有数据。

本机受控 HTTP 服务返回 `Set-Cookie` 的定向测试已通过：远程渲染写入 A 用户的 `_cookieJar`，A 用户可以读取并转入普通域名 Cookie，B 用户读不到；带路径的 URL 仍可按原域名读取，非法含路径的裸键仍被拒绝。测试使用独立临时工作目录，不读取或修改生产用户数据。

本机 JDK 11 执行 `./gradlew test bootJar --no-daemon` 成功；构建产物 `build/libs/reader-4.0.7.jar` 的 SHA-256 为 `40944B04C8421B12351D64BCCDC1C4302AEBF0B8FA6CBEA7B620D8C906772ADF`。这只是本机产物，不是 GitHub Actions 制品或线上版本。

## 尚未验证

- 真实需登录的 WebView 书源在原 JAR、当前远程服务和修复版上的全链路差分。原 JAR 的该缺陷属于有意修复，不能把结果差异误报为完全兼容。
- 浏览器自身 Cookie、重定向、SameSite、过期属性等更细的语义；当前 `CookieStore.cookieToMap` 对 `Set-Cookie` 属性的处理仍是旧实现，未在本次扩大修改。
- 生产部署。此次只读 SSH 连接未及时完成，未从服务器取书源或触碰线上服务。
