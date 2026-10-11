# 无凭据固定公开书籍的内置浏览器验收

日期：2026-10-11。当前为维护脚本候选；**尚未接受托管实站结果**。目标是在已独立验收的单 Reader 镜像内通过 Java/Kotlin API 实际读取固定公开书籍详情、目录和首章，不能只凭站点直连成功或测试夹具证明产品可用。

## 范围与依据

选择 Project Gutenberg 第84号《Frankenstein》，固定[书籍说明页](https://www.gutenberg.org/ebooks/84)及[官方在线正文](https://www.gutenberg.org/cache/epub/84/pg84-images.html)。[站点 robots](https://www.gutenberg.org/robots.txt)禁止 `/ebooks/search`，探针不请求搜索、批量遍历、账号、验证码、购买或其它私有资源。

本机直接 HTTPS 取样正文页面 HTTP200 / 468,370 B；两次 SHA 相同：23403a6a3b1cace42a06d6c378a6f1acf8ec97c940c652d12fe3c0d30275e58f。确认29个章节 div、固定目录起始锚点 letter1/letter2/letter3/letter4/chap01。这个站点直连/结构证据**不是 Reader 实际执行结果**，不把直连通过作为内置浏览器通过。

探针只用生成 Reader 账号、新隔离 storage 和保存后逐项读回的专用书源；从页面本身提取标题、作者、28项目录和第一个 Letter 正文，不注入伪造页面/元数据或空实现。三个业务 API 各尝试一次；失败立即停止后续业务，不重试或延长产品单请求超时。输出只有有限字段、状态、字符数、SHA、安全和资源断言，没有完整正文、匿名站点 Cookie 或原错误。

仍需原件/历史 WebView 同输入对照、真实站点认证和其它目标；本模式不声称补齐三方或真实登录。原件不上传 GitHub，不使用已清除的起点 Cookie，也不读取其它本地正文。

## 镜像身份与重复命令

使用 Native 成功任务38111391710、测试快照ccbdad907fb803ca4f75a38d182cb86009372dc9、JAR9b9d0c9808968c0390022156a5fcbfcc03f595ed228f124afbb06e830cd7dd84。维护脚本是后续独立 checkout；不改写输入镜像来源、拿输入镜像绿灯接受新探针，也不在本模式重建或上传 registry。

复用 `.github/workflows/browser-image.yml` 的显式 `gutenberg-fixed-book` 模式；普通 Full 和起点 metadata 模式不变。仅 GitHub 托管原生 runner、2 CPU / 2 GiB / 256 PID / 零swap、UID10001、cap-drop ALL、no-new-privileges、零宿主端口和私网拒绝。重新校验下载/载入/运行的准确归档、镜像与 JAR 身份，观察实际 worker/browser 进程；有成功外壳但缺书名/作者/目录/正文、资源失败、会话清理失败或未观察浏览器均判失败。

```powershell
gh workflow run browser-image.yml --repo warpdotsys/reader-dev --ref ci/full-reader-20260926 -f public_metadata_native_run=38111391710 -f public_metadata_revision=ccbdad907fb803ca4f75a38d182cb86009372dc9 -f public_metadata_capture=gutenberg-fixed-book -f native_arch=amd64
```

执行条件是指定原生 artifact 未过期。若原生版本已改变，脚本的准确 JAR 守卫必须先经审阅更新和另验，不能换入不明 `latest` 或修改期望以掩盖失败。

## 已知问题和回退

本机新增九项离线守卫通过；完整 Python636项 / 29.242秒 / 零失败 / 一个既有Windows POSIX跳过，Node119项全部通过，YAML解析和脚本语法检查通过。最初全量回归发现旧固定模式枚举断言未同步；更新为五个明确模式，继续检查原默认值、原起点分支及无插值传参，不移除安全断言。本机离线通过不作为本候选托管或实站业务通过。

当前源规则尚待本镜像实站执行，包括目录带 WebView 选项的 URL、原始页面片段章节以及 Rhino `chapter.url` 的真实解析。不能因九项离线守卫通过就称业务已通。站点页面和上游网络也是变量；单次失败不自动归因于 Reader，也不通过关 TLS、放开私网或处理验证码取绿灯。

此前起点、认证头、合并资源预算超时等失败仍保留；本次不发版、不合并或部署。回退只撤回新增显式模式和自有探针，旧起点模式及其失败记录保留；仅清理本轮自有容器和一次性 storage，不动生产或用户工作树。
