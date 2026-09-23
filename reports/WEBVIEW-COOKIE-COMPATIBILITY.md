# WebView Cookie 兼容修复

记录日期：2026-09-23。以下是源码与本机测试结论；尚未打发布标签或部署生产。

## 已从原始 JAR 验证

对只读备份 `reference/original/reader-pro-3.2.14.original.jar` 的 `RemoteWebview`、`CookieStore`、`NetworkUtils` 使用 JDK 11 `javap -c -p` 核对：远程渲染收到 `Set-Cookie` 后，把 `域名_cookieJar` 传给 `CookieStore.replaceCookie`；后者最终调用 `NetworkUtils.getSubDomain` 作为磁盘缓存键。`getSubDomain` 先调用 `getBaseUrl`，而 `getBaseUrl` 对不以 `http` 开头的字符串返回 `null`。因此这条 `_cookieJar` 写入路径在原 JAR 中无法产生缓存键。这是原版已有的缺陷，不是恢复工程新引入的差异。

## 已从当前源码确认并成功重建

`AnalyzeUrl` 在后续请求前会先读取 `域名_cookieJar`，再拷贝到普通域名 Cookie；旧实现对这两种非 URL 键都读不到。`CookieStore` 现在保留完整 URL 的原有域名归一化行为，同时接受受限字符集的裸域名和 `_cookieJar` 键；磁盘缓存仍沿用 ACache 及原有用户命名空间路径，不迁移既有数据。

本机受控 HTTP 服务返回 `Set-Cookie` 的定向测试已通过：远程渲染写入 A 用户的 `_cookieJar`，A 用户可以读取并转入普通域名 Cookie，B 用户读不到；带路径的 URL 仍可按原域名读取，非法含路径的裸键仍被拒绝。测试使用独立临时工作目录，不读取或修改生产用户数据。

补强测试先复现了当前源码将 `session=alpha==` 截为 `alpha` 的缺陷。现在 `Set-Cookie` 只把首个键值对存入 Cookie 缓存，不把 `Path`、`SameSite` 等响应属性当成下一次请求的 Cookie；按第一个 `=` 分割以保留值尾部的 `=`。`Max-Age=0`、空值和可解析的过去 `Expires` 会从 `_cookieJar` 与已复制的普通域名 Cookie 中删除对应键。受控书源的三次实际 HTTP 请求验证了“首次接收 → 第二次只发送 `session=alpha==` → 删除后第三次不发送”；远程 WebView 测试还覆盖了多键合并和用户隔离。这些是对恢复版的有意修复，不宣称与原 JAR 逐字节一致。

本机 JDK 11 执行 `./gradlew test bootJar --no-daemon` 成功；补强后的定向测试也通过。当前本机构建产物 `build/libs/reader-4.0.7.jar` 的 SHA-256 为 `6F6CAB12229CE6729B380F261F23183423337D1F4ABC9C30EB6FEC4ADA70A40E`。这不是 GitHub Actions 发布制品或线上版本，也不表示该 Gradle 配置已具备字节级可重复构建性。

## 尚未验证

- 真实需登录的 WebView 书源在原 JAR、当前远程服务和修复版上的全链路差分。原 JAR 的该缺陷属于有意修复，不能把结果差异误报为完全兼容。
- 浏览器自身的 Cookie、重定向、`Domain`/`Path` 限定、`SameSite` 策略、带引号和分号的复杂值，以及更多非标准日期格式。当前服务端 CookieStore 仍按原工程的简化“域名 → 键值串”模型工作，不是完整的 RFC 6265 浏览器 Cookie Jar。
- 生产部署。此次只读 SSH 连接未及时完成，未从服务器取书源或触碰线上服务。
